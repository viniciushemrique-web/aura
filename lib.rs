use std::sync::atomic::{AtomicU64, Ordering};

static CLIQUES: AtomicU64 = AtomicU64::new(0);
static TECLAS: AtomicU64 = AtomicU64::new(0);

/// Cliques e teclas globais (substitui pynput).
#[tauri::command]
fn contadores_entrada() -> (u64, u64) {
    (CLIQUES.load(Ordering::Relaxed), TECLAS.load(Ordering::Relaxed))
}

#[tauri::command]
fn alterar_volume(aumentar: bool) {
    #[cfg(windows)]
    win::tecla_midia(if aumentar { 0xAF } else { 0xAE }, 5); // VK_VOLUME_UP / DOWN
    #[cfg(not(windows))]
    let _ = aumentar;
}

fn executar_shutdown(args: &[&str]) -> Result<(), String> {
    let mut cmd = std::process::Command::new("shutdown");
    cmd.args(args);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    }
    match cmd.status() {
        Ok(s) if s.success() => Ok(()),
        Ok(s) => Err(format!("shutdown retornou {s}")),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
fn agendar_desligamento(minutos: u32) -> Result<(), String> {
    if minutos == 0 {
        return Err("minutos inválidos".into());
    }
    executar_shutdown(&["/s", "/t", &(minutos as u64 * 60).to_string()])
}

#[tauri::command]
fn cancelar_desligamento() -> Result<(), String> {
    executar_shutdown(&["/a"])
}

/// true se a janela em primeiro plano é do próprio app, do desktop/barra de tarefas,
/// invisível ou minimizada (a ilha só aparece nesses casos).
#[tauri::command]
fn primeiro_plano_e_area_de_trabalho() -> bool {
    #[cfg(windows)]
    {
        win::primeiro_plano_e_desktop()
    }
    #[cfg(not(windows))]
    {
        true
    }
}

#[cfg(windows)]
mod win {
    use windows::Win32::UI::Input::KeyboardAndMouse::{keybd_event, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP};
    use windows::Win32::UI::WindowsAndMessaging::{
        GetClassNameW, GetForegroundWindow, GetWindowThreadProcessId, IsIconic, IsWindowVisible,
    };

    const CLASSES: [&str; 6] = [
        "Progman", "WorkerW", "Shell_TrayWnd", "Shell_SecondaryTrayWnd",
        "NotifyIconOverflowWindow", "Windows.UI.Core.CoreWindow",
    ];

    pub fn tecla_midia(vk: u8, vezes: u8) {
        unsafe {
            for _ in 0..vezes {
                keybd_event(vk, 0, KEYBD_EVENT_FLAGS(0), 0);
                keybd_event(vk, 0, KEYEVENTF_KEYUP, 0);
            }
        }
    }

    pub fn primeiro_plano_e_desktop() -> bool {
        unsafe {
            let hwnd = GetForegroundWindow();
            if hwnd.is_invalid() {
                return true;
            }
            let mut pid = 0u32;
            GetWindowThreadProcessId(hwnd, Some(&mut pid));
            if pid == std::process::id() {
                return true;
            }
            let mut buf = [0u16; 256];
            let n = GetClassNameW(hwnd, &mut buf).max(0) as usize;
            if CLASSES.contains(&String::from_utf16_lossy(&buf[..n]).as_str()) {
                return true;
            }
            !IsWindowVisible(hwnd).as_bool() || IsIconic(hwnd).as_bool()
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    std::thread::spawn(|| {
        use rdev::{listen, EventType};
        let _ = listen(|e| match e.event_type {
            EventType::ButtonPress(_) => { CLIQUES.fetch_add(1, Ordering::Relaxed); }
            EventType::KeyPress(_) => { TECLAS.fetch_add(1, Ordering::Relaxed); }
            _ => {}
        });
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .invoke_handler(tauri::generate_handler![
            contadores_entrada,
            alterar_volume,
            agendar_desligamento,
            cancelar_desligamento,
            primeiro_plano_e_area_de_trabalho
        ])
        .run(tauri::generate_context!())
        .expect("erro ao iniciar o Tauri");
}
