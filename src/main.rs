use std::io::Write;
use std::collections::HashMap;
use winapi::um::fileapi::{GetLogicalDrives, GetDiskFreeSpaceA};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const KB_4_DATA: [u8; 4096] = [255u8; 4096];
const KB_1024_DATA: [u8; 1_048_576] = [255u8; 1_048_576];
static GB: u64 = 1_073_741_824;
static GB_70: u64 = GB * 70;
const DRIVE_LETTERS: [&str; 26] = ["A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R", "S", "T", "U", "V", "W", "X", "Y", "Z"];

fn main() {
    println!("By Adatan current version: {}", VERSION);
    let mut drive_letters = HashMap::new();
    let mut count = 0u8;
    let mut f = unsafe { GetLogicalDrives() };
    if f != 0 {
        println!("Список доступных дисков на выбранном устройстве:");
        for i in 0usize..32 {
            if f % 2 == 1 {
                count += 1;
                let drive_letter = format!(r"{}:\", DRIVE_LETTERS[i]);
                println!("{}) {}", count, &drive_letter);
                drive_letters.insert(count, drive_letter);
            }
            f /= 2;
        }
    }
    let drive_letter = drive_letters[&get_input("Выберите номер диска для заполнения: ").parse::<u8>().unwrap()].clone();
    println!("Выбран диск: {}", drive_letter);
    let mut sectors_per_cluster = 0u32;
    let mut bytes_per_sector = 0u32;
    let mut number_of_free_cluster = 0u32;
    let mut total_number_of_cluster = 0u32;
    if unsafe { GetDiskFreeSpaceA(#[allow(temporary_cstring_as_ptr)] std::ffi::CString::new(drive_letter.clone()).unwrap().as_ptr(), &mut sectors_per_cluster, &mut bytes_per_sector, &mut number_of_free_cluster, &mut total_number_of_cluster) } == 1 {
        let cluster_size = bytes_per_sector as u64 * sectors_per_cluster  as u64;
        let total_disk_space = cluster_size  as u64 * total_number_of_cluster  as u64;
        let free_disk_space = cluster_size  as u64 * number_of_free_cluster  as u64;
        println!("Размер кластера выбранного диска: {}", cluster_size);
        println!("Объем диска: {} гб.", total_disk_space / GB);
        println!("Объем доступной памяти: {} гб.", free_disk_space / GB);
        println!("Кол-во секторов на кластер: {}", sectors_per_cluster);
        println!("Кол-во байтов на сектор: {}", bytes_per_sector);
        println!("Кол-во свободных кластеров: {}", number_of_free_cluster);
        println!("Кол-во всех кластеров: {}", total_number_of_cluster);
        let data_for_file = match get_input("1) Заполнять каждый файл до 1 гб по 4 кб\n2) Заполнять каждый файл до 1 гб по 1024 кб").parse::<u8>().unwrap() {
            1 => &KB_4_DATA[..],
            2 => &KB_1024_DATA[..],
            _ => panic!("Только 1 или 2")
        };
        let file_path = format!(r"{}garbage\", drive_letter);
        std::fs::create_dir(file_path.clone()).unwrap();
        let mut total_write_size = 0u64;
        println!("Начинаю заполнение до 70 гб свободного остатка...");
        let start = get_unix_timestamp();
        while free_disk_space - total_write_size > GB_70 {
            let mut file_size = 0u64;
            let mut file_path = file_path.clone();
            file_path.push_str(get_unix_milli_timestamp().to_string().as_str());
            let mut file = std::fs::File::create(file_path).unwrap();
            while let Ok(b) = file.write(data_for_file) {
                if b == 0 {
                    break;
                }
                file_size += b as u64;
                if file_size >= GB {
                    break;
                }
            }
            total_write_size += file_size;
        }
        let end = get_unix_timestamp();
        println!("Заполнение длилось {} сек.", end - start);
        pause();
    }
}

fn get_unix_timestamp() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()
}

fn get_unix_milli_timestamp() -> u128 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()
}

//NOTE: Work only in Windows
fn pause() {
    use std::io::prelude::*;
    let mut stdin = std::io::stdin();
    let mut stdout = std::io::stdout();

    // We want the cursor to stay at the end of the line, so we print without a newline and flush manually.
    write!(stdout, "Press any key to continue...").unwrap();
    stdout.flush().unwrap();

    // Read a single byte and discard
    let _ = stdin.read(&mut [0u8]).unwrap();
}

fn get_input(prompt: &str) -> String {
    use std::io;
    println!("{}", prompt);
    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(_goes_into_input_above) => {},
        Err(_no_updates_is_fine) => {},
    }
    input.trim().to_string()
}