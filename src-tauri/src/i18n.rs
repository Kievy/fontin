//! Backend strings (errors, tray menu, notices) in pt-BR, English and Spanish.
//! The UI has its own dictionaries; this only covers text produced in Rust.

use std::sync::atomic::{AtomicU8, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    Pt,
    En,
    Es,
}

static CURRENT: AtomicU8 = AtomicU8::new(0);

impl Lang {
    /// `pref` is the saved setting: "system", "pt-BR", "en" or "es".
    pub fn resolve(pref: &str) -> Self {
        let tag = if pref.is_empty() || pref == "system" {
            sys_locale::get_locale().unwrap_or_default()
        } else {
            pref.to_owned()
        };
        let tag = tag.to_ascii_lowercase();
        if tag.starts_with("pt") {
            Lang::Pt
        } else if tag.starts_with("es") {
            Lang::Es
        } else if tag.is_empty() {
            Lang::Pt
        } else {
            Lang::En
        }
    }

    pub fn set_current(self) {
        CURRENT.store(self as u8, Ordering::Relaxed);
    }

    pub fn current() -> Self {
        match CURRENT.load(Ordering::Relaxed) {
            1 => Lang::En,
            2 => Lang::Es,
            _ => Lang::Pt,
        }
    }
}

/// Every user-facing message the backend can produce.
#[derive(Debug, Clone)]
pub enum Msg {
    UnsupportedPlatform,
    InvalidId,
    UnknownFace,
    UnknownFamily,
    FamilyWithoutFiles,
    DownloadInProgress,
    FolderNotFound,
    AlreadyIncluded(String),
    UnknownSource,
    GoogleNotRemovable,
    EmptyName,
    NoFontFiles,
    GoogleNoFiles(String),
    #[cfg_attr(not(windows), allow(dead_code))]
    WindowsRejected(String),
    Autostart(String),
    // Tray / notices
    TrayOpen,
    TrayDeactivateAll,
    TrayQuit,
    StylesDeactivated(usize),
}

impl Msg {
    pub fn text(&self) -> String {
        use Lang::*;
        let l = Lang::current();
        let pick = |pt: &str, en: &str, es: &str| match l {
            Pt => pt.to_owned(),
            En => en.to_owned(),
            Es => es.to_owned(),
        };
        match self {
            Msg::UnsupportedPlatform => pick(
                "plataforma não suportada",
                "unsupported platform",
                "plataforma no compatible",
            ),
            Msg::InvalidId => pick("id inválido", "invalid id", "id no válido"),
            Msg::UnknownFace => pick("estilo desconhecido", "unknown style", "estilo desconocido"),
            Msg::UnknownFamily => pick(
                "família desconhecida",
                "unknown family",
                "familia desconocida",
            ),
            Msg::FamilyWithoutFiles => pick(
                "família sem arquivos",
                "family has no files",
                "familia sin archivos",
            ),
            Msg::DownloadInProgress => pick(
                "download já em andamento",
                "download already in progress",
                "descarga ya en curso",
            ),
            Msg::FolderNotFound => pick(
                "pasta não encontrada",
                "folder not found",
                "carpeta no encontrada",
            ),
            Msg::AlreadyIncluded(n) => match l {
                Pt => format!("já incluída em “{n}”"),
                En => format!("already included in “{n}”"),
                Es => format!("ya incluida en “{n}”"),
            },
            Msg::UnknownSource => pick(
                "origem desconhecida",
                "unknown source",
                "origen desconocido",
            ),
            Msg::GoogleNotRemovable => pick(
                "o provedor Google não pode ser removido",
                "the Google provider can't be removed",
                "el proveedor de Google no se puede eliminar",
            ),
            Msg::EmptyName => pick("nome vazio", "empty name", "nombre vacío"),
            Msg::NoFontFiles => pick(
                "nenhum arquivo de fonte válido (.ttf, .otf, .ttc)",
                "no valid font files (.ttf, .otf, .ttc)",
                "ningún archivo de fuente válido (.ttf, .otf, .ttc)",
            ),
            Msg::GoogleNoFiles(f) => match l {
                Pt => format!("Google Fonts não retornou arquivos para {f}"),
                En => format!("Google Fonts returned no files for {f}"),
                Es => format!("Google Fonts no devolvió archivos para {f}"),
            },
            Msg::WindowsRejected(p) => match l {
                Pt => format!("o Windows recusou a fonte {p}"),
                En => format!("Windows rejected the font {p}"),
                Es => format!("Windows rechazó la fuente {p}"),
            },
            Msg::Autostart(e) => match l {
                Pt => format!("não foi possível alterar a inicialização automática: {e}"),
                En => format!("couldn't change launch at login: {e}"),
                Es => format!("no se pudo cambiar el inicio automático: {e}"),
            },
            Msg::TrayOpen => pick("Abrir Fontin", "Open Fontin", "Abrir Fontin"),
            Msg::TrayDeactivateAll => pick(
                "Desativar todas as fontes",
                "Deactivate all fonts",
                "Desactivar todas las fuentes",
            ),
            Msg::TrayQuit => pick("Sair", "Quit", "Salir"),
            Msg::StylesDeactivated(n) => match l {
                Pt => format!("{n} estilos desativados"),
                En => format!("{n} styles deactivated"),
                Es => format!("{n} estilos desactivados"),
            },
        }
    }
}
