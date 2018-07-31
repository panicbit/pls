extern crate ion_shell;
extern crate toml;
extern crate serde;
#[macro_use] extern crate failure;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate structopt;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use failure::Error;
use ion_shell::{ShellBuilder};
use structopt::StructOpt;
use std::env;

type Result<T> = ::std::result::Result<T, Error>;

#[derive(StructOpt, Debug)]
struct Opt {
    /// Tasks to run
    #[structopt(name = "TASKS")]
    tasks: Vec<String>,
}

fn main() {
    match do_main() {
        Ok(plz) => plz,
        Err(e) => {
            println!("{}", e);
            for cause in e.causes().skip(1) {
                println!("Caused by: {}", cause);
            }
        }
    }
}

fn do_main() -> Result<()> {
    let opt = Opt::from_args();
    let (path, plz) = load_plz_toml()?;

    env::set_current_dir(path)?;

    for task in opt.tasks {
        plz.run_task(&task)?;
    }

    Ok(())
}

fn load_plz_toml() -> Result<(PathBuf, Plz)> {
    use std::fs;

    let mut path = env::current_dir()?;

    loop {
        path.push("plz.toml");

        if path.is_file() {
            break;
        }

        path.pop();
        
        if !path.pop() {
            bail!("No 'plz.toml' found in current or any parent folder");
        }
    }

    let file = fs::read_to_string(&path)?;
    let plz = toml::from_str::<Plz>(&file)?;

    path.pop();

    Ok((path, plz))
}

#[derive(Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
struct Plz {
    tasks: HashMap<String, Arc<Task>>,
}

impl Plz {
    fn run_task(&self, task: &str) -> Result<()> {
        let task = self.tasks.get(task)
            .cloned()
            .ok_or_else(|| format_err!("Task '{}' does not exist", task))?;
        
        std::thread::spawn(move || {
            let mut shell = ShellBuilder::new().as_library();

            for cmd in &task.script {
                println!("+ {}", cmd);
                let code = shell.execute_command(cmd.as_str())?;

                if code != 0 {
                    bail!("Command exited with {}", code);
                }
            }

            Ok(())
        }).join().unwrap()
    }
}

#[derive(Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
struct Task {
    script: Vec<String>,
}
