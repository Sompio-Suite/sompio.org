//! What the app needs from the Sompio Lock, in one place: setting it up, unlocking, recovering,
//! and handing each app the key to its own encrypted database. The desktop shell only calls this.

use anyhow::{anyhow, bail, Context, Result};
use sompio_calendar::store::CalendarStore;
use sompio_clean::Store;
use sompio_connect::SecretStore;
use sompio_lock::{read_picture, LockFile, PictureSecret, VaultKey};
pub use sompio_lock::HardwareKeyInfo;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use zeroize::Zeroizing;

/// Where the picture's secret is kept on a computer the person chose to trust.
const REMEMBERED_PICTURE: &str = "lock.picture";
/// Calendar's own key, in the computer's credential store. Calendar works without the Sompio
/// Lock: it's protected by the person's computer account, like a browser's or mail app's data.
const CALENDAR_KEY: &str = "calendar.key";

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct LockStatus {
    pub set_up: bool,
    pub unlocked: bool,
    /// This computer remembers the key picture, so only the password is needed
    pub remembers_picture: bool,
    /// YubiKeys added as a stand-in for the key picture
    #[serde(default)]
    pub hardware_keys: Vec<sompio_lock::HardwareKeyInfo>,
    /// Calendar can be opened right now (it doesn't need the lock on this computer)
    #[serde(default)]
    pub calendar_open: bool,
}

pub struct Vault {
    dir: PathBuf,
    secrets: Arc<dyn SecretStore>,
    key: Mutex<Option<VaultKey>>,
    /// First run: the key picture made before the password is chosen
    pending_picture: Mutex<Option<PictureSecret>>,
}

impl Vault {
    pub fn new(dir: PathBuf, secrets: Arc<dyn SecretStore>) -> Self {
        Self { dir, secrets, key: Mutex::new(None), pending_picture: Mutex::new(None) }
    }

    pub fn lock_path(&self) -> PathBuf {
        self.dir.join("lock.json")
    }

    pub fn clean_db_path(&self) -> PathBuf {
        self.dir.join("clean.db")
    }

    fn set_key(&self, v: Option<VaultKey>) {
        if let Ok(mut k) = self.key.lock() {
            *k = v;
        }
    }

    fn remembered(&self) -> Option<PictureSecret> {
        self.secrets.get(REMEMBERED_PICTURE).ok().flatten().and_then(|h| PictureSecret::from_hex(&h))
    }

    pub fn status(&self) -> Result<LockStatus> {
        Ok(LockStatus {
            set_up: self.lock_path().exists(),
            unlocked: self.key.lock().map(|k| k.is_some()).unwrap_or(false),
            remembers_picture: self.remembered().is_some(),
            hardware_keys: LockFile::load(&self.lock_path()).ok().flatten().map(|f| f.hardware_keys()).unwrap_or_default(),
            calendar_open: self.device_calendar_key().is_some() || self.calendar_can_start_fresh(),
        })
    }

    fn picture_from(&self, path: Option<&Path>) -> Result<PictureSecret> {
        match path {
            Some(p) => read_picture(&std::fs::read(p).with_context(|| format!("Couldn't open {}", p.display()))?),
            None => self.remembered().ok_or_else(|| anyhow!("Choose your key picture. This computer doesn't remember it.")),
        }
    }

    fn remember(&self, picture: &PictureSecret, remember: bool) -> Result<()> {
        if remember {
            self.secrets.set(REMEMBERED_PICTURE, &picture.to_hex())
        } else {
            self.secrets.delete(REMEMBERED_PICTURE)
        }
    }

    /// Brings existing app data under the lock: encrypts a plain database in place, or moves aside
    /// one that belongs to a lock that no longer exists (it can't be read by anyone).
    fn secure_data(&self, vault: &VaultKey) -> Result<()> {
        // Calendar: moved to its own key, with a backup only the lock can open. Problems here
        // never stop the unlock; Calendar then simply keeps needing the lock.
        let _ = self.settle_calendar_key(vault);
        let db = self.clean_db_path();
        if !db.exists() {
            return Ok(());
        }
        let key = vault.derive("clean.db");
        if Store::is_plain(&db) {
            Store::encrypt_existing(&db, &key)?;
            self.remove_old_veil_copy();
        } else if Store::open_encrypted(&db, &key).is_err() {
            let aside = self.dir.join(format!("clean.db.unreadable-{}", sompio_lock::now_ms()));
            std::fs::rename(&db, aside)?;
        }
        Ok(())
    }

    /// The plain copy left behind by the app's old name, now that the data is encrypted here.
    fn remove_old_veil_copy(&self) {
        if let Some(old) = self.dir.parent().map(|p| p.join("app.veil.desktop")) {
            for f in ["veil.db", "veil.db-wal", "veil.db-shm"] {
                let _ = std::fs::remove_file(old.join(f));
            }
        }
    }

    /// First run: creates the lock, writes the key picture where the person chose, and returns the
    /// recovery code to show once.
    pub fn setup(&self, password: &str, picture_path: &Path, remember: bool) -> Result<Zeroizing<String>> {
        if self.lock_path().exists() {
            bail!("Sompio is already locked with a password. Unlock it instead.");
        }
        let s = LockFile::create(password)?;
        std::fs::write(picture_path, &s.picture_png)
            .with_context(|| format!("Couldn't save the key picture to {}", picture_path.display()))?;
        // The lock file is saved before any data is encrypted, so data is never encrypted under a
        // key that isn't written down anywhere.
        s.file.save(&self.lock_path())?;
        self.remember(&s.picture_secret, remember)?;
        self.secure_data(&s.vault)?;
        self.set_key(Some(s.vault.clone()));
        Ok(s.recovery_code)
    }

    /// Setup without a key picture file: the picture's secret is made and kept only in this
    /// computer's credential store, so the password alone opens Sompio here. The recovery code is
    /// the way back in on another computer, or if the credential store is ever cleared.
    pub fn setup_without_picture(&self, password: &str) -> Result<Zeroizing<String>> {
        if self.lock_path().exists() {
            bail!("Sompio is already locked with a password. Unlock it instead.");
        }
        let s = LockFile::create(password)?;
        // Kept first and read back: without it, only the recovery code could open the lock.
        self.remember(&s.picture_secret, true)
            .map_err(|_| anyhow!("This computer can't keep the key safely. Tick \"Also use a key picture\" instead."))?;
        if self.remembered().map(|p| p.to_hex()) != Some(s.picture_secret.to_hex()) {
            bail!("This computer can't keep the key safely. Tick \"Also use a key picture\" instead.");
        }
        s.file.save(&self.lock_path())?;
        self.secure_data(&s.vault)?;
        self.set_key(Some(s.vault.clone()));
        Ok(s.recovery_code)
    }

    /// Saves the key picture this computer remembers as a file, for a copy off the computer or to
    /// unlock on another one. Needs Sompio unlocked.
    pub fn export_picture(&self, picture_path: &Path) -> Result<()> {
        if !self.status()?.unlocked {
            bail!("Unlock Sompio first.");
        }
        let secret = self.remembered().ok_or_else(|| anyhow!("This computer doesn't remember a key picture."))?;
        let png = sompio_lock::make_picture(&secret)?;
        std::fs::write(picture_path, &png).with_context(|| format!("Couldn't save the key picture to {}", picture_path.display()))?;
        let back = std::fs::read(picture_path).with_context(|| format!("Couldn't read back {}", picture_path.display()))?;
        if read_picture(&back)?.as_bytes() != secret.as_bytes() {
            bail!("The key picture didn't save correctly. Try another folder.");
        }
        Ok(())
    }

    /// First run, step one: makes the key picture, saves it where the person chose, and reads it
    /// back to be sure the saved file works. The password comes next (`setup_with_picture`).
    pub fn new_picture(&self, picture_path: &Path) -> Result<()> {
        if self.lock_path().exists() {
            bail!("Sompio is already locked with a password. Unlock it instead.");
        }
        let secret = PictureSecret::new()?;
        let png = sompio_lock::make_picture(&secret)?;
        std::fs::write(picture_path, &png).with_context(|| format!("Couldn't save the key picture to {}", picture_path.display()))?;
        let back = std::fs::read(picture_path).with_context(|| format!("Couldn't read back {}", picture_path.display()))?;
        if read_picture(&back)?.as_bytes() != secret.as_bytes() {
            bail!("The key picture didn't save correctly. Try another folder.");
        }
        *self.pending_picture.lock().map_err(|_| anyhow!("lock"))? = Some(secret);
        Ok(())
    }

    /// First run, step two: the password, with the key picture made in step one.
    pub fn setup_with_picture(&self, password: &str, remember: bool) -> Result<Zeroizing<String>> {
        if self.lock_path().exists() {
            bail!("Sompio is already locked with a password. Unlock it instead.");
        }
        let picture = self
            .pending_picture
            .lock()
            .map_err(|_| anyhow!("lock"))?
            .clone()
            .ok_or_else(|| anyhow!("Make your key picture first."))?;
        let s = LockFile::create_with_picture(password, picture)?;
        s.file.save(&self.lock_path())?;
        self.remember(&s.picture_secret, remember)?;
        self.secure_data(&s.vault)?;
        self.set_key(Some(s.vault.clone()));
        if let Ok(mut p) = self.pending_picture.lock() {
            *p = None;
        }
        Ok(s.recovery_code)
    }

    pub fn unlock(&self, password: &str, picture_path: Option<&Path>, remember: bool) -> Result<()> {
        let file = LockFile::load(&self.lock_path())?.ok_or_else(|| anyhow!("Sompio has no lock yet."))?;
        let picture = self.picture_from(picture_path)?;
        let vault = file.unlock(password, &picture)?;
        if picture_path.is_some() {
            self.remember(&picture, remember)?;
        }
        self.secure_data(&vault)?;
        self.set_key(Some(vault));
        Ok(())
    }

    /// Recovery code in, new password and new key picture out. Returns the new recovery code.
    /// Without `picture_path`, the new picture's secret is kept only on this computer, as in
    /// `setup_without_picture`.
    pub fn recover(&self, code: &str, new_password: &str, picture_path: Option<&Path>, remember: bool) -> Result<Zeroizing<String>> {
        let file = LockFile::load(&self.lock_path())?.ok_or_else(|| anyhow!("Sompio has no lock yet."))?;
        let s = file.recover(code, new_password)?;
        match picture_path {
            Some(p) => {
                std::fs::write(p, &s.picture_png).with_context(|| format!("Couldn't save the new key picture to {}", p.display()))?;
                self.remember(&s.picture_secret, remember)?;
            }
            None => {
                self.remember(&s.picture_secret, true)
                    .map_err(|_| anyhow!("This computer can't keep the key safely. Choose to make a new key picture instead."))?;
            }
        }
        s.file.save(&self.lock_path())?;
        self.secure_data(&s.vault)?;
        self.set_key(Some(s.vault.clone()));
        Ok(s.recovery_code)
    }

    /// Asks each added YubiKey in turn (via `respond`, which talks to the device) until one
    /// answers, and turns its answer into the picture's secret.
    fn picture_from_hardware(&self, file: &LockFile, respond: &mut dyn FnMut(&sompio_lock::HardwareKeyInfo) -> Result<[u8; 20]>) -> Result<PictureSecret> {
        let keys = file.hardware_keys();
        if keys.is_empty() {
            bail!("No YubiKey has been added to this Sompio yet.");
        }
        let mut last = None;
        for k in &keys {
            match respond(k).and_then(|r| file.picture_from_hardware(k.index, &r)) {
                Ok(p) => return Ok(p),
                Err(e) => last = Some(e),
            }
        }
        Err(last.unwrap_or_else(|| anyhow!("No added YubiKey answered.")))
    }

    /// Password plus a YubiKey instead of the picture.
    pub fn unlock_with_hardware(&self, password: &str, respond: &mut dyn FnMut(&sompio_lock::HardwareKeyInfo) -> Result<[u8; 20]>) -> Result<()> {
        let file = LockFile::load(&self.lock_path())?.ok_or_else(|| anyhow!("Sompio has no lock yet."))?;
        let picture = self.picture_from_hardware(&file, respond)?;
        let vault = file.unlock(password, &picture)?;
        self.secure_data(&vault)?;
        self.set_key(Some(vault));
        Ok(())
    }

    /// Adds a YubiKey. Needs Sompio unlocked, and the key picture (or this computer remembering it).
    /// `respond` asks the device; it returns the answer and the key's serial number.
    pub fn add_hardware_key(
        &self,
        picture_path: Option<&Path>,
        slot: u8,
        label: &str,
        respond: &mut dyn FnMut(u8, &[u8; 32]) -> Result<([u8; 20], Option<u32>)>,
    ) -> Result<()> {
        if !self.status()?.unlocked {
            bail!("Unlock Sompio first.");
        }
        let mut file = LockFile::load(&self.lock_path())?.ok_or_else(|| anyhow!("Sompio has no lock yet."))?;
        let picture = self.picture_from(picture_path)?;
        let challenge = LockFile::new_challenge()?;
        let (response, serial) = respond(slot, &challenge)?;
        // Ask again: a key whose slot doesn't give the same answer twice can't open anything later
        let (again, _) = respond(slot, &challenge)?;
        if again != response {
            bail!("The YubiKey gave two different answers. Its slot {slot} needs to be set up for HMAC-SHA1 challenge-response.");
        }
        file.add_hardware_key(&picture, challenge, response, slot, serial, label)?;
        file.save(&self.lock_path())
    }

    pub fn remove_hardware_key(&self, index: usize) -> Result<()> {
        if !self.status()?.unlocked {
            bail!("Unlock Sompio first.");
        }
        let mut file = LockFile::load(&self.lock_path())?.ok_or_else(|| anyhow!("Sompio has no lock yet."))?;
        file.remove_hardware_key(index)?;
        file.save(&self.lock_path())
    }

    /// Changing the password with a YubiKey instead of the picture.
    pub fn change_password_with_hardware(&self, old: &str, new: &str, respond: &mut dyn FnMut(&sompio_lock::HardwareKeyInfo) -> Result<[u8; 20]>) -> Result<()> {
        let mut file = LockFile::load(&self.lock_path())?.ok_or_else(|| anyhow!("Sompio has no lock yet."))?;
        let picture = self.picture_from_hardware(&file, respond)?;
        file.change_password(old, new, &picture)?;
        file.save(&self.lock_path())
    }

    pub fn change_password(&self, old: &str, new: &str, picture_path: Option<&Path>) -> Result<()> {
        let mut file = LockFile::load(&self.lock_path())?.ok_or_else(|| anyhow!("Sompio has no lock yet."))?;
        let picture = self.picture_from(picture_path)?;
        file.change_password(old, new, &picture)?;
        file.save(&self.lock_path())
    }

    pub fn forget_picture(&self) -> Result<()> {
        self.secrets.delete(REMEMBERED_PICTURE)
    }

    pub fn lock(&self) {
        self.set_key(None);
    }

    /// Clean's database and its key. Fails while Sompio is locked.
    pub fn clean_db(&self) -> Result<(PathBuf, Zeroizing<[u8; 32]>)> {
        let guard = self.key.lock().map_err(|_| anyhow!("lock state unavailable"))?;
        let vault = guard.as_ref().ok_or_else(|| anyhow!("Sompio is locked. Unlock it to continue."))?;
        Ok((self.clean_db_path(), vault.derive("clean.db")))
    }

    pub fn vault_db_path(&self) -> PathBuf {
        self.dir.join("vault.kdbx")
    }

    /// The password vault's file and key. Fails while Sompio is locked.
    pub fn vault_db(&self) -> Result<(PathBuf, Zeroizing<[u8; 32]>)> {
        let guard = self.key.lock().map_err(|_| anyhow!("lock state unavailable"))?;
        let vault = guard.as_ref().ok_or_else(|| anyhow!("Sompio is locked. Unlock it to continue."))?;
        Ok((self.vault_db_path(), vault.derive("vault.kdbx")))
    }

    /// Voice: the master identity (each server gets its own key made from it), and the key
    /// and path for saved servers. Fails while Sompio is locked.
    pub fn voice_keys(&self) -> Result<(Zeroizing<[u8; 32]>, Zeroizing<[u8; 32]>, PathBuf)> {
        let guard = self.key.lock().map_err(|_| anyhow!("lock state unavailable"))?;
        let vault = guard.as_ref().ok_or_else(|| anyhow!("Sompio is locked. Unlock it to continue."))?;
        Ok((vault.derive("voice.identity"), vault.derive("voice.json"), self.dir.join("voice.dat")))
    }

    pub fn data_dir(&self) -> PathBuf {
        self.dir.clone()
    }

    pub fn calendar_db_path(&self) -> PathBuf {
        self.dir.join("calendar.db")
    }

    fn calendar_backup_path(&self) -> PathBuf {
        self.dir.join("calendar.key")
    }

    fn device_calendar_key(&self) -> Option<Zeroizing<[u8; 32]>> {
        let hex = Zeroizing::new(self.secrets.get(CALENDAR_KEY).ok().flatten()?);
        if hex.len() != 64 {
            return None;
        }
        let mut k = Zeroizing::new([0u8; 32]);
        for i in 0..32 {
            k[i] = u8::from_str_radix(hex.get(i * 2..i * 2 + 2)?, 16).ok()?;
        }
        Some(k)
    }

    fn store_calendar_key(&self, key: &[u8; 32]) -> Result<()> {
        let hex = Zeroizing::new(key.iter().map(|b| format!("{b:02x}")).collect::<String>());
        self.secrets.set(CALENDAR_KEY, &hex)?;
        // Read back: a credential store that silently drops it would lose the calendar
        match self.device_calendar_key() {
            Some(k) if *k == *key => Ok(()),
            _ => bail!("This computer's credential store didn't keep Calendar's key."),
        }
    }

    /// A brand-new Calendar (no file, no backup) can start with its own key right away.
    fn calendar_can_start_fresh(&self) -> bool {
        !self.calendar_db_path().exists() && !self.calendar_backup_path().exists()
    }

    /// Gives Calendar its own key, kept in the computer's credential store, with a backup sealed by
    /// the lock. Moves a calendar that was encrypted under the lock, and restores the key from the
    /// backup if the credential store lost it. Returns the key to use, or the lock's key for
    /// Calendar when this computer can't keep one (Calendar then keeps needing the lock).
    fn settle_calendar_key(&self, vault: &VaultKey) -> Result<Zeroizing<[u8; 32]>> {
        let db = self.calendar_db_path();
        let backup = self.calendar_backup_path();
        let seal_with = vault.derive("calendar.key.backup");
        let old = vault.derive("calendar.db");
        let write_backup = |k: &[u8; 32]| -> Result<()> {
            if !backup.exists() {
                std::fs::write(&backup, sompio_lock::seal_key(&seal_with, k)?)?;
            }
            Ok(())
        };

        if let Some(k) = self.device_calendar_key() {
            // A move that stopped halfway: the file still opens with the lock's key
            if db.exists() && !CalendarStore::opens_with(&db, &k) && CalendarStore::opens_with(&db, &old) {
                write_backup(&k)?;
                CalendarStore::rekey(&db, &old, &k)?;
            }
            write_backup(&k)?;
            return Ok(k);
        }
        if backup.exists() {
            if let Some(k) = sompio_lock::open_key(&seal_with, &std::fs::read_to_string(&backup)?) {
                let _ = self.store_calendar_key(&k);
                return Ok(k);
            }
        }
        let new = sompio_lock::random_key()?;
        if db.exists() && !CalendarStore::opens_with(&db, &old) {
            // Neither key opens it: leave it alone rather than guess
            return Ok(old);
        }
        if self.store_calendar_key(&new).is_err() {
            return Ok(old);
        }
        let _ = std::fs::remove_file(&backup);
        write_backup(&new)?;
        if db.exists() {
            if let Err(e) = CalendarStore::rekey(&db, &old, &new) {
                // Put things back the way they were, so the lock's key keeps working
                let _ = self.secrets.delete(CALENDAR_KEY);
                let _ = std::fs::remove_file(&backup);
                return Err(e);
            }
        }
        Ok(new)
    }

    /// Calendar's database and its key. Works without the lock once Calendar has its own key on
    /// this computer; before that (or where the computer can't keep a key) it needs Sompio unlocked.
    pub fn calendar_db(&self) -> Result<(PathBuf, Zeroizing<[u8; 32]>)> {
        if let Some(k) = self.device_calendar_key() {
            if !self.calendar_db_path().exists() || CalendarStore::opens_with(&self.calendar_db_path(), &k) {
                return Ok((self.calendar_db_path(), k));
            }
        }
        {
            let guard = self.key.lock().map_err(|_| anyhow!("lock state unavailable"))?;
            if let Some(vault) = guard.as_ref() {
                return Ok((self.calendar_db_path(), self.settle_calendar_key(vault)?));
            }
        }
        if self.calendar_can_start_fresh() {
            let new = sompio_lock::random_key()?;
            if self.store_calendar_key(&new).is_ok() {
                return Ok((self.calendar_db_path(), new));
            }
        }
        bail!("Sompio is locked. Unlock it to continue.")
    }
}

#[cfg(test)]
mod hardware_tests {
    use super::*;

    struct Mem(std::sync::Mutex<std::collections::HashMap<String, String>>);
    impl SecretStore for Mem {
        fn get(&self, k: &str) -> Result<Option<String>> { Ok(self.0.lock().unwrap().get(k).cloned()) }
        fn set(&self, k: &str, v: &str) -> Result<()> { self.0.lock().unwrap().insert(k.into(), v.into()); Ok(()) }
        fn delete(&self, k: &str) -> Result<()> { self.0.lock().unwrap().remove(k); Ok(()) }
    }

    #[test]
    fn picture_first_then_password() {
        let dir = tempfile::tempdir().unwrap();
        let v = Vault::new(dir.path().to_path_buf(), Arc::new(Mem(Default::default())));
        let pic = dir.path().join("key.png");
        assert!(v.setup_with_picture("correct horse battery staple", false).is_err(), "needs the picture first");
        v.new_picture(&pic).unwrap();
        assert!(pic.exists());
        let code = v.setup_with_picture("correct horse battery staple", false).unwrap();
        assert!(!code.is_empty());
        v.lock();
        assert!(v.unlock("correct horse battery staple", None, false).is_err(), "picture not remembered");
        v.unlock("correct horse battery staple", Some(&pic), false).unwrap();
        assert!(v.new_picture(&dir.path().join("again.png")).is_err(), "already set up");
    }

    #[test]
    fn unlock_with_a_yubikey() {
        let dir = tempfile::tempdir().unwrap();
        let v = Vault::new(dir.path().to_path_buf(), Arc::new(Mem(Default::default())));
        let pic = dir.path().join("key.png");
        v.setup("correct horse battery staple", &pic, false).unwrap();
        // A pretend YubiKey: HMAC is faked as a hash of the challenge with a device secret
        let device = |ch: &[u8; 32]| -> [u8; 20] { let mut r = [0u8; 20]; for i in 0..20 { r[i] = ch[i] ^ 0x5a ^ ch[31 - i]; } r };
        v.add_hardware_key(Some(&pic), 2, "Blue key", &mut |_, ch| Ok((device(ch), Some(777)))).unwrap();
        let st = v.status().unwrap();
        assert_eq!(st.hardware_keys.len(), 1);
        assert_eq!(st.hardware_keys[0].label, "Blue key");
        v.lock();
        assert!(v.unlock_with_hardware("wrong password here!", &mut |k| Ok(device(&k.challenge))).is_err());
        assert!(v.unlock_with_hardware("correct horse battery staple", &mut |_| Ok([0; 20])).is_err(), "another key");
        v.unlock_with_hardware("correct horse battery staple", &mut |k| Ok(device(&k.challenge))).unwrap();
        assert!(v.status().unwrap().unlocked);
        assert!(v.clean_db().is_ok());
        v.change_password_with_hardware("correct horse battery staple", "new horse battery staple!", &mut |k| Ok(device(&k.challenge))).unwrap();
        v.lock();
        v.unlock_with_hardware("new horse battery staple!", &mut |k| Ok(device(&k.challenge))).unwrap();
        v.remove_hardware_key(0).unwrap();
        assert!(v.status().unwrap().hardware_keys.is_empty());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sompio_connect::MemoryStore;

    const PW: &str = "fell lantern quiet 42";

    fn vault(dir: &Path) -> Vault {
        Vault::new(dir.to_path_buf(), Arc::new(MemoryStore::default()))
    }

    #[test]
    fn setup_encrypts_existing_data_and_unlock_flows() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("app.sompio.desktop");
        std::fs::create_dir_all(&dir).unwrap();
        // Data from before the lock, plus the old app's plain copy
        {
            let mut s = Store::open(&dir.join("clean.db")).unwrap();
            sompio_clean::load_demo(&mut s).unwrap();
        }
        let old = tmp.path().join("app.veil.desktop");
        std::fs::create_dir_all(&old).unwrap();
        std::fs::copy(dir.join("clean.db"), old.join("veil.db")).unwrap();

        let v = vault(&dir);
        assert_eq!(v.status().unwrap(), LockStatus { set_up: false, unlocked: false, remembers_picture: false, hardware_keys: vec![], calendar_open: true });
        assert!(v.clean_db().is_err());
        let pic = tmp.path().join("Sompio key.png");
        let code = v.setup(PW, &pic, true).unwrap();
        assert!(pic.exists());
        assert!(!Store::is_plain(&dir.join("clean.db")), "existing data is encrypted");
        assert!(!old.join("veil.db").exists(), "old plain copy removed");
        let (path, key) = v.clean_db().unwrap();
        assert_ne!(*v.calendar_db().unwrap().1, *key, "each app has its own key");
        assert!(Store::open_encrypted(&path, &key).unwrap().has_data().unwrap());
        assert!(v.setup(PW, &pic, true).is_err(), "setup only once");

        // Lock, then unlock with the password alone (picture remembered)
        v.lock();
        assert!(v.clean_db().is_err());
        assert!(v.unlock("wrong password here", None, true).is_err());
        v.unlock(PW, None, true).unwrap();
        assert!(v.status().unwrap().unlocked);

        // A new computer: nothing remembered, so the picture is needed
        let fresh = vault(&dir);
        assert!(fresh.unlock(PW, None, false).unwrap_err().to_string().contains("Choose your key picture"));
        fresh.unlock(PW, Some(&pic), false).unwrap();
        assert!(!fresh.status().unwrap().remembers_picture);

        // Recovery: new password and picture, same data
        let fresh2 = vault(&dir);
        let pic2 = tmp.path().join("Sompio key 2.png");
        let code2 = fresh2.recover(&code, "brand new phrase here", Some(&pic2), true).unwrap();
        assert_ne!(code2.as_str(), code.as_str());
        let (path, key) = fresh2.clean_db().unwrap();
        assert!(Store::open_encrypted(&path, &key).unwrap().has_data().unwrap());
        assert!(vault(&dir).unlock(PW, Some(&pic), false).is_err(), "old password and picture stop working");
        vault(&dir).unlock("brand new phrase here", Some(&pic2), false).unwrap();
    }

    fn schedules(path: &Path, key: &[u8; 32]) -> usize {
        CalendarStore::open_encrypted(path, key).unwrap().load().unwrap().len()
    }

    #[test]
    fn calendar_works_without_the_lock_and_survives_it() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().to_path_buf();
        let v = vault(&dir);
        // Before any lock: Calendar opens and keeps what's added
        assert!(v.status().unwrap().calendar_open);
        let (path, key) = v.calendar_db().unwrap();
        CalendarStore::open_encrypted(&path, &key).unwrap().create_shared("Team", "Europe/Amsterdam", &[]).unwrap();
        assert_eq!(schedules(&path, &key), 2);
        assert!(v.clean_db().is_err(), "Trace still needs the lock");

        // Setting up the lock keeps Calendar's key, and seals a backup of it
        v.setup_without_picture(PW).unwrap();
        assert!(dir.join("calendar.key").exists());
        v.lock();
        let (_, again) = v.calendar_db().unwrap();
        assert_eq!(*again, *key, "same key while locked");
        assert_eq!(schedules(&path, &again), 2);

        // The credential store loses the key: locked, Calendar waits for the lock; unlocking restores it
        v.secrets.delete(CALENDAR_KEY).unwrap();
        assert!(v.calendar_db().is_err());
        assert!(!v.status().unwrap().calendar_open);
        v.unlock(PW, None, false).unwrap();
        v.lock();
        let (_, restored) = v.calendar_db().unwrap();
        assert_eq!(*restored, *key);
    }

    #[test]
    fn a_calendar_under_the_old_lock_moves_to_its_own_key() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().to_path_buf();
        let v = vault(&dir);
        v.setup(PW, &dir.join("k.png"), true).unwrap();
        // How it was before: Calendar encrypted with the lock's key, no key of its own
        let old = v.key.lock().unwrap().as_ref().unwrap().derive("calendar.db");
        v.secrets.delete(CALENDAR_KEY).unwrap();
        std::fs::remove_file(dir.join("calendar.key")).unwrap();
        let path = dir.join("calendar.db");
        let _ = std::fs::remove_file(&path);
        CalendarStore::open_encrypted(&path, &old).unwrap().create_shared("Team", "Europe/Amsterdam", &[]).unwrap();
        v.lock();
        assert!(v.calendar_db().is_err(), "the old calendar needs one unlock to move");

        v.unlock(PW, None, true).unwrap();
        v.lock();
        let (_, key) = v.calendar_db().unwrap();
        assert_ne!(*key, *old);
        assert!(!CalendarStore::opens_with(&path, &old));
        assert_eq!(schedules(&path, &key), 2, "nothing lost in the move");
    }

    #[test]
    fn lock_without_a_picture_file() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().to_path_buf();
        let v = vault(&dir);
        let code = v.setup_without_picture(PW).unwrap();
        assert!(v.status().unwrap().remembers_picture);
        assert!(!std::fs::read_dir(&dir).unwrap().any(|e| e.unwrap().path().extension().is_some_and(|x| x == "png")));
        v.lock();
        v.unlock(PW, None, false).unwrap();
        // A picture file can be made later, and works on another computer
        let pic = dir.join("later.png");
        v.export_picture(&pic).unwrap();
        vault(&dir).unlock(PW, Some(&pic), false).unwrap();
        // Recovery without a new picture: this computer keeps the new key
        let other = vault(&dir);
        other.recover(&code, "another good phrase", None, false).unwrap();
        other.lock();
        other.unlock("another good phrase", None, false).unwrap();
    }

    #[test]
    fn data_from_a_lost_lock_is_moved_aside() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().to_path_buf();
        {
            let mut s = Store::open(&dir.join("clean.db")).unwrap();
            sompio_clean::load_demo(&mut s).unwrap();
        }
        Store::encrypt_existing(&dir.join("clean.db"), &[9u8; 32]).unwrap();
        let v = vault(&dir);
        v.setup(PW, &dir.join("k.png"), false).unwrap();
        assert!(!dir.join("clean.db").exists());
        assert!(std::fs::read_dir(&dir).unwrap().any(|e| e.unwrap().file_name().to_string_lossy().starts_with("clean.db.unreadable-")));
    }
}
