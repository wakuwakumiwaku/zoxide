use anyhow::{Result, bail};

use crate::cmd::{Remove, Run};
use crate::db::Database;
use crate::{config, util};

impl Run for Remove {
    fn run(&self) -> Result<()> {
        let mut db = Database::open()?;

        for path in &self.paths {
            if db.remove(path) {
                continue;
            }
            let path_abs = util::resolve_path(path)?;
            if db.remove(util::path_to_str(&path_abs)?) {
                continue;
            }
            // Match add's storage policy, but only after literal/lexical
            // lookup: stored paths must remain removable after the
            // directory disappears.
            if config::resolve_symlinks()
                && let Ok(path_resolved) = util::canonicalize(path)
                && db.remove(util::path_to_str(&path_resolved)?)
            {
                continue;
            }
            bail!("path not found in database: {path}");
        }

        db.save()
    }
}
