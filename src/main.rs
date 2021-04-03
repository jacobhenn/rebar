mod workspaces;
mod time;
mod battery;

use crate::workspaces::Workspaces;
use crate::time::Time;
use crate::battery::Battery;
use std::time::Duration;
use chrono::Timelike;
use tokio::io::{BufWriter, BufReader, AsyncBufReadExt, AsyncWriteExt};
use tokio::process::Command;
use std::process::Stdio;
use std::error::Error;
use tokio::time::timeout;
use std::fmt;

const ONE_SECOND: Duration = Duration::from_secs(1);

struct Bar {
    time: Time,
    workspaces: Workspaces,
    battery: Battery,
}

impl fmt::Display for Bar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, " {}   {}^pa(1814){}", self.time, self.workspaces, self.battery)
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

impl Bar {
    async fn update_workspaces(&mut self) -> Result<(), Box<dyn Error>> {
        self.workspaces = hc_tag_status().await?;
        self.pass_second().await?;
        Ok(())
    }

    async fn pass_second(&mut self) -> Result<(), Box<dyn Error>> {
        self.time.0 = self.time.0 + chrono::Duration::from_std(ONE_SECOND)?;
        if self.time.0.second() == 0 {
            self.battery.update_charge().await?;
        }
        Ok(())
    }

    async fn new() -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            time: Time::new(),
            workspaces: hc_tag_status().await?,
            battery: Battery::new().await?,
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // set up herbstclient
    let mut hc_cmd = Command::new("herbstclient");
    hc_cmd.arg("--idle").arg("tag_changed");
    hc_cmd.stdout(Stdio::piped());
    let mut hc_child = hc_cmd.spawn()?;
    let hc_stdout = hc_child.stdout.take().expect("couldn't get hc's stdout");
    let mut hc_stdout_reader = BufReader::new(hc_stdout).lines();

    // set up dzen2
    let mut dz_cmd = Command::new("dzen2");
    dz_cmd.arg("-fn").arg("-*-firacode-*-r-normal-*-*-105-*-*-*-*-iso8859-*");
    dz_cmd.arg("-ta").arg("l");
    dz_cmd.arg("-bg").arg("#2e3440");
    dz_cmd.arg("-fg").arg("#2e3440");
    dz_cmd.arg("-h").arg("22");
    dz_cmd.stdin(Stdio::piped());
    let mut dz_child = dz_cmd.spawn()?;
    let dz_stdin = dz_child.stdin.take().expect("couldn't get dzen's stdin");
    let mut dz_stdin_writer = BufWriter::new(dz_stdin);

    let mut bar = Bar::new().await?;

    loop {
        dz_stdin_writer.write(&bar.to_string().as_bytes()).await?;
        dz_stdin_writer.write_u8(b'\n').await?;
        dz_stdin_writer.flush().await?;
        if timeout(Duration::from_secs(1), hc_stdout_reader.next_line()).await.is_ok() {
            bar.update_workspaces().await?;
        } else {
            bar.pass_second().await?;
        }
    }
}
