#[cfg(target_os = "macos")]
pub fn check_case_sensitivity(volume: &str) -> Result<bool, io::Error> {
    let first_path = format!("{}/carrier_test", volume);
    let second_path = format!("{}/carrierVM_test", volume);
    {
        let mut first = File::create(&first_path)?;
        first.write_all(b"first")?;
    }
    {
        let mut second = File::create(&second_path)?;
        second.write_all(b"second")?;
    }
    let mut data = String::new();
    {
        let mut test = File::open(&first_path)?;

        test.read_to_string(&mut data)?;
    }
    if data == "first" {
        let _ = std::fs::remove_file(first_path);
        let _ = std::fs::remove_file(second_path);
        Ok(true)
    } else {
        let _ = std::fs::remove_file(first_path);
        Ok(false)
    }
}

#[cfg(target_os = "macos")]
pub fn check_volume(cfg: &mut CarrierConfig) {
    if !cfg.storage_volume.is_empty() {
        return;
    }

    println!(
        "
On macOS, carrier requires a dedicated, case-sensitive volume.
You can easily create such volume by executing something like
this on another terminal:

diskutil apfs addVolume disk3 \"Case-sensitive APFS\" carrier

NOTE: APFS volume creation is a non-destructive action that
doesn't require a dedicated disk nor \"sudo\" privileges. The
new volume will share the disk space with the main container
volume.
"
    );
    loop {
        print!("Please enter the mountpoint for this volume [/Volumes/carrier]: ");
        io::stdout().flush().unwrap();
        let answer: String = read!("{}\n");

        let volume = if answer.is_empty() {
            "/Volumes/carrier".to_string()
        } else {
            answer.to_string()
        };

        print!("Checking volume... ");
        match check_case_sensitivity(&volume) {
            Ok(res) => {
                if res {
                    println!("success.");
                    println!("The volume has been configured. Please execute carrier again");
                    cfg.storage_volume = volume;
                    confy::store(APP_NAME, cfg).unwrap();
                    std::process::exit(-1);
                } else {
                    println!("failed.");
                    println!("This volume failed the case sensitivity test.");
                }
            }
            Err(err) => {
                println!("error.");
                println!("There was an error running the test: {}", err);
            }
        }
    }
}

#[cfg(target_os = "linux")]
pub fn check_unshare() {
    let uid = unsafe { libc::getuid() };
    if uid != 0
        && std::env::vars()
        .find(|(key, _)| key == "BUILDAH_ISOLATION")
        .is_none()
    {
        println!("Please re-run carrier inside a \"buildah unshare\" session");
        std::process::exit(-1);
    }
}