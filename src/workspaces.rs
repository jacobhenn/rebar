use std::{fmt, str};
use std::error::Error;
use tokio::process::Command;

pub enum Workspace {
    Empty,
    Unfocused { name: String },
    Focused { name: String },
    Urgent { name: String },
}

impl fmt::Display for Workspace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, ""),
            Self::Unfocused { name } => write!(f, "^fg(#88c0d0){}^fg()", name),
            Self::Focused { name } => write!(f, "^bg(#bf616a) {} ^bg()", name),
            Self::Urgent { name } => write!(f, "^bg(#ebcb8b) {} ^bg()", name),
        }
    }
}

#[derive(Debug)]
pub enum ParseWorkspaceError {
    Empty,
    UnknownPrefix { prefix: char },

}

impl fmt::Display for ParseWorkspaceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "UNREACHABLE ERROR")
    }
}

impl Error for ParseWorkspaceError {
    fn source(&self) -> Option<&(dyn Error + 'static)> { None }
}

impl str::FromStr for Workspace {
    type Err = ParseWorkspaceError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.chars();
        if let Some(prefix) = chars.next() {
            match prefix {
                '.' => Ok(Workspace::Empty),
                ':' => Ok(Workspace::Unfocused { name: chars.as_str().to_string() }),
                '#' => Ok(Workspace::Focused { name: chars.as_str().to_string() }),
                '!' => Ok(Workspace::Urgent { name: chars.as_str().to_string() }),
                _ => Err(ParseWorkspaceError::UnknownPrefix { prefix }),
            }
        } else { Err(ParseWorkspaceError::Empty) }
    }
}

pub struct Workspaces(Vec<Workspace>);

impl fmt::Display for Workspaces {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self(workspaces) = self;
        let s = workspaces
            .iter()
            .map(|w| w.to_string())
            .filter(|w| w.len() != 0)
            .collect::<Vec<_>>().join(" ");
        write!(f, "{}", s)
    }
}

impl str::FromStr for Workspaces {
    type Err = ParseWorkspaceError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let words = s.trim().split('\t');
        let workspaces: Vec<Workspace> = words
            .map(|w| w.parse())
            .collect::<Result<Vec<Workspace>, ParseWorkspaceError>>()?;
        Ok(Self(workspaces))
    }
}

async fn hc_tag_status() -> Result<Workspaces, Box<dyn Error>> {
    let buf = Command::new("herbstclient")
        .arg("tag_status")
        .output()
        .await?
        .stdout;

    Ok(String::from_utf8(buf)?.parse()?)

}

impl Workspaces {
    pub async fn new() -> Result<Self, Box<dyn Error>> {
        Ok(hc_tag_status().await?)
    }

    pub async fn update(&mut self) -> Result<(), Box<dyn Error>> {
        *self = hc_tag_status().await?;
        Ok(())
    }
}
