use clap::{App, Arg};

static DEFAULT_UPLOAD_SIZE: u64 = 100;
static MAX_UPLOAD_SIZE: u64 = 1000;
static MIN_UPLOAD_SIZE: u64 = 10;
static DEFAULT_NUM_UPLOADERS: u64 = 2;
static DEFAULT_WATCHER_INTERVAL: u64 = 2;
static MIN_WATCHER_INTERVAL: u64 = 1;

pub struct Config {
    watched_dirs: Vec<String>,
    bucket_name: String,
    num_uploaders: u64,
    upload_part_size: u64,
    watcher_delay: u64,
}

impl Config {
    pub fn from_args() -> Self {
        let upload_size_default = format!("{}", DEFAULT_UPLOAD_SIZE);
        let watcher_interval_default = format!("{}", DEFAULT_WATCHER_INTERVAL);
        let uploader_threads_default = format!("{}", DEFAULT_NUM_UPLOADERS);
        let matches = App::new("S3 File Sync")
            .version("0.0.1")
            .author("Vlad Vasiliu")
            .about("Sync directories to S3")
            .arg(
                Arg::with_name("watch_dir")
                    .short("w")
                    .long("watch-dir")
                    .value_name("DIR")
                    .help("Directories to watch")
                    .takes_value(true)
                    .required(true)
                    .min_values(1)
                    .multiple(true),
            )
            .arg(
                Arg::with_name("watch_interval")
                    .short("i")
                    .long("watcher-interval")
                    .value_name("DURATION")
                    .help(&format!(
                        "Seconds between file change notifications. Must be at least {}",
                        MIN_WATCHER_INTERVAL
                    ))
                    .takes_value(true)
                    .required(false)
                    .default_value(&watcher_interval_default)
                    .validator(int_gte_1),
            )
            .arg(
                Arg::with_name("bucket_name")
                    .short("b")
                    .long("bucket")
                    .value_name("BUCKET")
                    .help("AWS bucket name")
                    .takes_value(true)
                    .required(true),
            )
            .arg(
                Arg::with_name("upload_size")
                    .short("s")
                    .long("upload-part-size")
                    .value_name("SIZE")
                    .help(&format!(
                        "Upload part size in MB. Must be between {} and {}",
                        MIN_UPLOAD_SIZE, MAX_UPLOAD_SIZE
                    ))
                    .takes_value(true)
                    .required(false)
                    .default_value(&upload_size_default)
                    .validator(upload_size_between_bounds),
            )
            .arg(
                Arg::with_name("uploader_threads")
                    .short("u")
                    .long("uploader-threads")
                    .value_name("NUM")
                    .help("Number of uploader threads")
                    .takes_value(true)
                    .required(false)
                    .default_value(&uploader_threads_default)
                    .validator(int_gte_1),
            )
            .get_matches();

        Self {
            watched_dirs: matches
                .values_of("watch_dir")
                .unwrap()
                .map(|e| e.into())
                .collect(),
            bucket_name: matches.value_of("bucket_name").unwrap().into(),
            num_uploaders: matches
                .value_of("uploader_threads")
                .unwrap()
                .parse()
                .unwrap(),
            upload_part_size: matches.value_of("upload_size").unwrap().parse().unwrap(),
            watcher_delay: matches.value_of("watch_interval").unwrap().parse().unwrap(),
        }
    }

    pub fn pretty_string(&self) -> String {
        let mut result = String::from("Supplied configuration:\n");
        result.push_str("\tUploader:\n");
        result.push_str(&format!("\t\tBucket name:\t{}\n", self.bucket_name));
        result.push_str(&format!("\t\tThreads:\t{}\n", self.num_uploaders));
        result.push_str(&format!("\t\tPart size:\t{} MB\n", self.upload_part_size));
        result.push_str("\tWatcher:\n");
        result.push_str(&format!("\t\tDelay:\t\t{}s\n", self.watcher_delay));
        result.push_str("\t\tDirectories:\n");

        for dir in &self.watched_dirs {
            result.push_str(&format!("\t\t\t- {}\n", dir));
        }

        result
    }
}

fn int_gte_1(num: String) -> Result<(), String> {
    match num.parse::<u64>().or_else(|err| Err(format!("{}", err)))? {
        x if x < 1 => Err("Must be greater than or equal to 1.".into()),
        _ => Ok(()),
    }
}

fn upload_size_between_bounds(num: String) -> Result<(), String> {
    match num.parse::<u64>().or_else(|err| Err(format!("{}", err)))? {
        x if (x >= MIN_UPLOAD_SIZE) && (x <= MAX_UPLOAD_SIZE) => Ok(()),
        _ => Err(format!(
            "Upload size must be between {} and {} MB",
            MIN_UPLOAD_SIZE, MAX_UPLOAD_SIZE
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::{int_gte_1, upload_size_between_bounds, MAX_UPLOAD_SIZE, MIN_UPLOAD_SIZE};

    use proptest::prelude::*;

    #[test]
    fn test_int_gte_1_works_for_1_to_10() {
        for x in 1..10 {
            let num = format!("{}", x);
            assert_eq!(int_gte_1(num).is_ok(), true)
        }
    }

    #[test]
    fn test_int_gte_1_breaks_for_ints_under_1() {
        for x in -10..1 {
            let num = format!("{}", x);
            assert_eq!(int_gte_1(num).is_err(), true)
        }
    }

    #[test]
    fn test_int_gte_1_breaks_for_strings() {
        assert_eq!(int_gte_1("test".into()).is_err(), true)
    }

    #[test]
    fn test_upload_size_works_for_valid_values() {
        for x in MIN_UPLOAD_SIZE..=MAX_UPLOAD_SIZE {
            let num = format!("{}", x);
            assert_eq!(upload_size_between_bounds(num).is_ok(), true);
        }
    }

    #[test]
    fn test_upload_size_breaks_for_text() {
        assert_eq!(upload_size_between_bounds("test".into()).is_err(), true);
    }

    #[test]
    fn test_upload_size_breaks_for_out_of_bounds() {
        for x in &[MIN_UPLOAD_SIZE - 1, MAX_UPLOAD_SIZE + 1] {
            let num = format!("{}", x);
            assert_eq!(upload_size_between_bounds(num).is_err(), true);
        }
    }

    #[test]
    fn int_gte_1_doesnt_crash() {
        proptest!(|(s in "\\PC*")| {
            int_gte_1(s).ok();
        } );
    }

    #[test]
    fn upload_size_between_bounds_doesnt_crash() {
        proptest!(|(s in "\\PC*")| {
            upload_size_between_bounds(s).ok();
        } );
    }
}
