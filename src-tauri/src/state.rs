use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;

use parking_lot::Mutex;

use crate::aura::controller::AuraController;
use crate::config::ConfigStore;
use crate::error::{NoCrateError, Result};
use crate::sio::SioMonitor;
use crate::wmi::connection::WmiConnection;

/// A request to execute on the WMI thread.
type WmiRequest = Box<dyn FnOnce(&WmiConnection) + Send>;

/// Thread-safe handle to the dedicated WMI thread.
///
/// Because COM objects (IWbemServices) are not Send/Sync, we run all WMI
/// operations on a single dedicated thread and communicate via channels.
pub struct WmiThread {
    sender: mpsc::Sender<WmiRequest>,
}

impl WmiThread {
    /// Spawn the dedicated WMI thread and establish the COM connection.
    ///
    /// # Errors
    ///
    /// Returns an error if the WMI connection fails during initialization.
    pub fn spawn() -> Result<Self> {
        let (init_tx, init_rx) = mpsc::channel::<std::result::Result<(), NoCrateError>>();
        let (req_tx, req_rx) = mpsc::channel::<WmiRequest>();

        let _handle = thread::Builder::new()
            .name("nocrate-wmi".into())
            .spawn(move || {
                // Attempt to create the WMI connection on this thread
                let conn = match WmiConnection::new() {
                    Ok(c) => {
                        let _ = init_tx.send(Ok(()));
                        c
                    }
                    Err(e) => {
                        let _ = init_tx.send(Err(e));
                        return;
                    }
                };

                // Process requests until the channel is closed
                for request in req_rx {
                    request(&conn);
                }

                // `conn` drops here → CoUninitialize on this thread
            })
            .map_err(|e| NoCrateError::Unknown(format!("Failed to spawn WMI thread: {e}")))?;

        // Wait for initialization result
        init_rx
            .recv()
            .map_err(|_| NoCrateError::Wmi("WMI thread died during init".into()))??;

        Ok(Self { sender: req_tx })
    }

    /// Execute a closure on the WMI thread and receive the result.
    ///
    /// The closure runs on the dedicated WMI thread with access to the
    /// `WmiConnection`. The result is sent back via a oneshot channel.
    ///
    /// # Errors
    ///
    /// Returns an error if the WMI thread is dead or the closure returns an error.
    pub fn execute<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&WmiConnection) -> Result<T> + Send + 'static,
        T: Send + 'static,
    {
        let (tx, rx) = mpsc::channel();

        let request: WmiRequest = Box::new(move |conn| {
            let result = f(conn);
            let _ = tx.send(result);
        });

        self.sender
            .send(request)
            .map_err(|_| NoCrateError::Wmi("WMI thread is no longer running".into()))?;

        rx.recv()
            .map_err(|_| NoCrateError::Wmi("WMI thread did not respond".into()))?
    }
}

/// Application state managed by Tauri.
///
/// Holds shared resources accessible from all commands.
pub struct AppState {
    /// WMI thread handle.
    /// `None` if the WMI connection failed (e.g. no admin or no ASUS drivers).
    pub wmi: Option<WmiThread>,
    /// AURA controller behind a Mutex (HidDevice is Send but not Sync).
    /// `None` if no controller was found at startup.
    pub aura: Mutex<Option<AuraController>>,
    /// Persistent configuration store.
    pub config: ConfigStore,
    /// If WMI initialization failed, the error message is stored here
    /// so the frontend can show a meaningful explanation.
    pub wmi_error: Option<String>,
    /// Super I/O 传感器监控器（读取风扇 RPM 和温度）
    /// `None` 表示驱动加载失败或未检测到支持的芯片
    pub sio: Option<SioMonitor>,
    /// SIO 初始化失败时的错误信息
    pub sio_error: Option<String>,
}

impl AppState {
    /// Create a new `AppState` by initializing all subsystems.
    ///
    /// WMI and AURA discovery failures are both non-fatal — the app
    /// launches regardless, with degraded functionality.
    pub fn new(app_data_dir: PathBuf, resource_dir: PathBuf) -> Result<Self> {
        let (wmi, wmi_error) = match WmiThread::spawn() {
            Ok(w) => (Some(w), None),
            Err(e) => {
                eprintln!("Warning: WMI initialization failed: {e}");
                eprintln!("Fan control features will be unavailable.");
                (None, Some(e.to_string()))
            }
        };

        let aura = match AuraController::discover() {
            Ok(ctrl) => {
                eprintln!("AURA controller found: {:?}", ctrl.info());
                Some(ctrl)
            }
            Err(e) => {
                eprintln!("AURA controller not found: {e}");
                None
            }
        };

        // 初始化 Super I/O 传感器监控（非致命）
        let (sio, sio_error) = match SioMonitor::init(&resource_dir) {
            Ok(m) => (Some(m), None),
            Err(e) => {
                eprintln!("Warning: SIO initialization failed: {e}");
                (None, Some(e.to_string()))
            }
        };

        let config = ConfigStore::init(app_data_dir)?;

        Ok(Self {
            wmi,
            aura: Mutex::new(aura),
            config,
            wmi_error,
            sio,
            sio_error,
        })
    }
}
