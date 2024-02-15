mod transpile;
mod ui;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ui = ui::App::build_and_run()?;
    Ok(())
}
