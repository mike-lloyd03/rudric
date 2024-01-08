#[cfg(test)]
mod integration {
    use anyhow::{bail, Result};
    use rexpect::{process::wait, spawn};

    #[test]
    fn test_init() -> Result<()> {
        let test_dir = "testdata/test_init";
        let _ = std::fs::remove_dir_all(test_dir);
        std::fs::create_dir_all(test_dir)?;

        let process_str = format!("target/debug/rudric -c {test_dir} init");
        let mut p = spawn(&process_str, Some(10_000))?;
        p.exp_regex("Set master password")?;
        p.send_line("password")?;
        p.exp_regex("Confirm password")?;
        p.send_line("password")?;

        match p.process.wait() {
            Ok(wait::WaitStatus::Exited(_, 0)) => (),
            Ok(wait::WaitStatus::Exited(_, c)) => {
                bail!("failed with exit code {c}: {}", p.exp_eof()?)
            }
            _ => bail!("Other error"),
        }

        std::fs::remove_dir_all(test_dir)?;

        Ok(())
    }
}
