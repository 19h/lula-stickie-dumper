use std::{env, fs};
use std::path::Path;

use chrono::NaiveDate;
use walkdir::WalkDir;
use crate::rtftotext::convert;

mod rtftotext;
mod rtf_control;

fn main() {
    // find timemachine backup directories on external volumes
    let mut backup_dirs = Vec::new();
    let mut legacy_backup_dirs = Vec::new();

    for entry in WalkDir::new("/Volumes").max_depth(2) {
        let entry =
            if let Ok(entry) = entry {
                entry
            } else {
                let entry = &entry.unwrap_err();
                println!("Error reading {}: {:?}", entry.path().unwrap().display(), entry.io_error().unwrap().kind());

                continue;
            };

        if !entry.file_type().is_dir() {
            continue;
        }

        let entry_fn = entry.file_name().to_str().unwrap();
        let entry_path = entry.path().to_str().unwrap().to_string();

        if entry_fn.starts_with("Backups.backupdb") {
            legacy_backup_dirs.push(entry_path.clone());
        }

        if entry_fn.ends_with(".inprogress") || entry_fn.ends_with(".previous") || entry_fn.ends_with(".interrupted") {
            // check if entry_path + .com.apple.timemachine.checkpoint exists
            let checkpoint_path = entry_path.clone() + "/.com.apple.timemachine.checkpoint";

            if Path::new(&checkpoint_path).exists() {
                backup_dirs.push(entry_path.clone());
            }
        }
    }

    println!("Found {} backup directories", backup_dirs.len());
    println!("Found {} legacy backup directories", legacy_backup_dirs.len());

    println!("");

    for dir in backup_dirs.iter() {
        println!("Found TimeMachine location: {}", dir);
    }

    println!("");

    for dir in legacy_backup_dirs.iter() {
        println!("Found legacy TimeMachine location: {}", dir);
    }

    println!("");

    // find the most recent backup
    let mut backup_roots = Vec::<String>::new();

    for backup_dir in backup_dirs {
        for entry in WalkDir::new(&backup_dir).max_depth(1) {
            if let Ok(entry) = entry {
                let path = entry.path().to_str().unwrap().to_string();

                if !entry.file_type().is_dir() {
                    continue;
                }

                if path != backup_dir {
                    backup_roots.push(path);
                }
            }
        }
    }

    for backup_dir in legacy_backup_dirs {
        for entry in WalkDir::new(&backup_dir).max_depth(1) {
            if let Ok(entry) = entry {
                let path = entry.path().to_str().unwrap().to_string();

                if !entry.file_type().is_dir() {
                    continue;
                }

                if path == backup_dir {
                    continue;
                }

                let backup_path = &path;

                for entry in WalkDir::new(&path).max_depth(1) {
                    if let Ok(entry) = entry {
                        let path = entry.path().to_str().unwrap().to_string();

                        if !entry.file_type().is_dir() {
                            continue;
                        }

                        if &path == backup_path {
                            continue;
                        }

                        let dated_backup_path = &path;

                        for entry in WalkDir::new(&dated_backup_path).max_depth(1) {
                            if let Ok(entry) = entry {
                                let path = entry.path().to_str().unwrap().to_string();

                                if !entry.file_type().is_dir() {
                                    continue;
                                }

                                if &path == dated_backup_path {
                                    continue;
                                }

                                backup_roots.push(path);
                            }
                        }
                    }
                }
            }
        }
    }

    if backup_roots.len() == 0 {
        println!("No backups found");

        return;
    }

    for backup_root in backup_roots.iter() {
        println!("Found backup root: {}", backup_root);
    }

    println!("");

    let mut user_folders = Vec::<String>::new();

    for backup_root in backup_roots.iter() {
        // try to find users in the backup_root + /Users folder

        let user_folder = backup_root.clone() + "/Users";

        for entry in WalkDir::new(&user_folder).max_depth(1) {
            if let Ok(entry) = entry {
                let path = entry.path().to_str().unwrap().to_string();

                if !entry.file_type().is_dir() {
                    continue;
                }

                if path == user_folder {
                    continue;
                }

                user_folders.push(path);
            }
        }
    }

    if user_folders.len() == 0 {
        println!("No user folders found");

        return;
    }

    for user_folder in user_folders.iter() {
        println!("Found user folders: {}", user_folder);
    }

    println!("");

    let mut stickies_db_paths = Vec::<String>::new();

    for user_folder in user_folders.iter() {
        let stickies_db_path = user_folder.clone() + "/Library/Containers/com.apple.stickies/Data/Library/Stickies";

        if Path::new(&stickies_db_path).exists() {
            stickies_db_paths.push(stickies_db_path);
        }
    }

    if stickies_db_paths.len() == 0 {
        println!("No Stickies databases found");

        return;
    }

    for stickies_db_path in stickies_db_paths.iter() {
        println!("Found Stickies database: {}", stickies_db_path);
    }

    println!("");

    let mut notes = Vec::<(String, String)>::new();

    for stickies_db_path in stickies_db_paths.iter() {
        for entry in WalkDir::new(&stickies_db_path).max_depth(2) {
            if let Ok(entry) = entry {
                let path = entry.path().to_str().unwrap().to_string();

                if !entry.file_type().is_dir() {
                    continue;
                }

                if !path.ends_with(".rtfd") {
                    continue;
                }

                let mut note_path = path.clone() + "/TXT.rtf";

                if Path::new(&note_path).exists() {
                    println!("Found note: {}", &note_path);

                    // path file name
                    notes.push(
                        (
                            Path::new(&path).file_name().unwrap().to_str().unwrap().to_string(),
                            note_path
                        )
                    );
                }
            }
        }
    }

    if notes.len() == 0 {
        println!("No notes found");

        return;
    }

    println!("");

    // Create a lula-notes folder on the desktop
    let mut desktop_path = env::home_dir().unwrap();
    desktop_path.push("Desktop");
    desktop_path.push("lula-notes");

    if !desktop_path.exists() {
        println!("Creating lula-notes folder on the desktop: {}", desktop_path.to_str().unwrap());

        fs::create_dir(&desktop_path).unwrap();
    }

    println!("");

    for note in notes.iter() {
        let note_name = &note.0;
        let note_path = &note.1;

        let mut outpath = desktop_path.clone();
        outpath.push(note_name.clone());

        let outpath_rtf = outpath.clone().with_extension("rtf");

        // copy note_path to outpath
        println!("Copying note: {} to {}", note_path, outpath_rtf.to_str().unwrap());

        fs::copy(note_path, &outpath_rtf).unwrap();

        let outpath_txt = outpath.clone().with_extension("txt");

        if let Err(_) = convert(
            Some(&note_path),
            Some(&outpath_txt.to_str().unwrap().to_string()),
        ) {
            println!("Error converting note from rtf to txt: {}", note_name);
        };
    }
}
