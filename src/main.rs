//! Binary entry point for the Ulti terminal emulator.

use anyhow::Result;

fn main() -> Result<()> {
    ulti::app::run()
}
