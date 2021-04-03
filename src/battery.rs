use tokio::fs;
use std::fmt;
use std::error::Error;

async fn get_charge() -> Result<u8, Box<dyn Error>> {
    let contents = fs::read("/sys/class/power_supply/BAT0/charge_now").await?;
    let raw_charge: u32 = String::from_utf8_lossy(&contents).trim().parse()?;
    Ok((raw_charge / 34870) as u8)
}

pub struct Battery(pub u8);

impl fmt::Display for Battery {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self(charge) = self;
        let charge_difference = 100 - charge;
        write!(
            f,
            "^fg(#ebcb8b)^r({}x10)^fg()^fg(#4c566a)^r({}x10)^fg()",
            charge,
            charge_difference
        )
    }
}

impl Battery {
    pub async fn update_charge(&mut self) -> Result<(), Box<dyn Error>> {
        self.0 = get_charge().await?;
        Ok(())
    }

    pub async fn new() -> Result<Self, Box<dyn Error>> {
        Ok(Self(get_charge().await?))
    }
}
