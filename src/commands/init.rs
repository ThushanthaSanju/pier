use anyhow::Result;

use crate::shell::{init_snippet, ShellKind};

pub fn run(shell: ShellKind) -> Result<()> {
    print!("{}", init_snippet(shell));
    Ok(())
}
