use std::io::Write;
use std::collections::HashMap;
use std::fs::{create_dir, remove_dir_all};
use std::fs::File;
use std::ffi::CString;
use winapi::um::fileapi::{GetLogicalDrives, GetDiskFreeSpaceA};
use dirs::desktop_dir;

const VERSION: &str = env!("CARGO_PKG_VERSION");
#[allow(non_upper_case_globals)]
static GiB: u64 = 1_073_741_824;

enum HowToFill {
    WriteAll,
    WriteByChunk
}

fn main() {
    println!("By Adatan. current version: {}", VERSION);
    let mut drive_letters = HashMap::new();
    let mut count = 0usize;
    let mut f = unsafe { GetLogicalDrives() };
    if f == 0 {
        panic!("Не обнаружено дисков");
    }
    println!("Список доступных дисков на выбранном устройстве:");
    for i in 0usize..32 {
        if f % 2 == 1 {
            count += 1;
            let drive_letter = format!(r"{}:\", char::from(65 + i as u8));
            println!("{}) {}", count, &drive_letter);
            drive_letters.insert(count, drive_letter);
        }
        f /= 2;
    }
    count = 0;
    let mut drive_letter = get_input("Выберите номер диска или букву диска (пример: С) для заполнения: ");
    drive_letter = match drive_letter.parse::<usize>() {
        Ok(drive_number) => drive_letters.get(&drive_number).unwrap().clone(),
        Err(_) => format!(r"{}:\", drive_letter.to_uppercase())
    };
    println!("Выбран диск: {}", drive_letter.clone());
    let desktop_dir = desktop_dir().unwrap();
    let file_path = if desktop_dir.starts_with(drive_letter.clone()) {
        format!(r"{}\garbage\", desktop_dir.to_str().unwrap())
    } else {
        format!(r"{}garbage\", drive_letter.clone())
    };
    let mut sectors_per_cluster = 0u32;
    let mut bytes_per_sector = 0u32;
    let mut number_of_free_cluster = 0u32;
    let mut total_number_of_cluster = 0u32;
    if unsafe { GetDiskFreeSpaceA(#[allow(temporary_cstring_as_ptr)] CString::new(drive_letter).unwrap().as_ptr(), &mut sectors_per_cluster, &mut bytes_per_sector, &mut number_of_free_cluster, &mut total_number_of_cluster) } != 1 {
        panic!("err: winapi func call GetDiskFreeSpaceA is finished not successfully")
    }
    let cluster_size = bytes_per_sector as u64 * sectors_per_cluster as u64;
    let total_disk_space = cluster_size as u64 * total_number_of_cluster as u64;
    let free_disk_space = cluster_size as u64 * number_of_free_cluster as u64;
    println!("Размер кластера выбранного диска: {} б", cluster_size);
    println!("Объем диска: {} GiB", total_disk_space / GiB);
    println!("Объем доступной памяти: {} GiB", free_disk_space / GiB);
    println!("Кол-во секторов на кластер: {}", sectors_per_cluster);
    println!("Кол-во байтов на сектор: {} B", bytes_per_sector);
    println!("Кол-во свободных кластеров: {}", number_of_free_cluster);
    println!("Кол-во всех кластеров: {}", total_number_of_cluster);
    let (total_garbage_size, kind) = get_bytes_size("Введите какой объем диска нужно заполнить(пример: 1024Б, 1024KB, 1024MB, 1024GB, 1024KiB, 1024MiB, 1024GiB): ");
    if total_garbage_size > free_disk_space {
        panic!("err: trying to fill more than available ");
    }
    let how_to_fill = match get_input("1) Делать записи по чанкам\n2) Записывать сразу весь файл").as_str() {
        "1" => HowToFill::WriteByChunk,
        "2" => HowToFill::WriteAll,
        _ => panic!("err: unknown method of fill")
    };
    let coefficient = get_coefficient(&kind);

    let mut garbage_file_size = match how_to_fill {
        HowToFill::WriteAll => get_input(format!("По сколько {} будет каждый файл?", &kind).as_str()),
        HowToFill::WriteByChunk => get_input(format!("По сколько {} будет каждый чанк для заполнения каждого файла?", &kind).as_str())
    }.parse::<u64>().unwrap();
    garbage_file_size *= coefficient;
    if total_garbage_size % garbage_file_size != 0 {
        panic!("Ошибка: невозможно заполнения без остатка");
    }
    let garbage_data = vec![0u8; garbage_file_size as usize];
    println!("Пытаюсь создать папку для хранения бессмысленных данных...");
    if let Err(_) = create_dir(file_path.clone()) {
        println!("Возможно, папка с файлами уже существует, пробуем удалить папку со всеми файлами...");
        if let Ok(_) = remove_dir_all(file_path.clone()) {
            println!("Папка удалена, создаем новую...");
            create_dir(file_path.clone()).unwrap();
            println!("Папка создана")
        }
    };
    let mut stdout = std::io::stdout();
    let start = get_unix_timestamp();
    let mut total_wrote_size = 0u64;
    match how_to_fill {
        HowToFill::WriteByChunk => {
            while total_wrote_size < total_garbage_size {
                let mut file_size = 0u64;
                let mut file_path = file_path.clone();
                file_path.push_str(count.to_string().as_str());
                let mut file = File::create(file_path).unwrap();
                while let Ok(b) = file.write(&garbage_data) {
                    if b == 0 {
                        break;
                    }
                    file_size += b as u64;
                    if file_size >= garbage_file_size {
                        break;
                    }
                }
                total_wrote_size += file_size;
                print!("\r{}/{} {}", total_wrote_size / coefficient, total_garbage_size / coefficient, &kind);
                stdout.flush().unwrap();
                count += 1;
            }
        },
        HowToFill::WriteAll => {
            while total_wrote_size < total_garbage_size {
                let mut file_path = file_path.clone();
                file_path.push_str(count.to_string().as_str());
                let mut file = File::create(file_path).unwrap();
                file.write_all(&garbage_data).unwrap();
                total_wrote_size += garbage_file_size;
                print!("\r{}/{} {}", total_wrote_size / coefficient, total_garbage_size / coefficient, &kind);
                stdout.flush().unwrap();
                count += 1;
            }
        },
    };
    let end = get_unix_timestamp();
    println!("\r\nЗаполнение длилось {} сек.", end - start);
    pause();
}

fn get_unix_timestamp() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()
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

fn get_bytes_size(prompt: &str) -> (u64, String) {
    let chunk_size = get_input(prompt);
    let mut number = String::new();
    let mut kind = String::new();
    for c in chunk_size.chars() {
        if c.is_numeric() {
            number.push(c);
        } else {
            kind.push(c)
        }
    }
    let kind = kind.trim().to_uppercase();
    let number = number.trim().parse::<u64>().unwrap();
    let coefficient = get_coefficient(&kind);
    (number * coefficient, kind)
}

fn get_coefficient(kind: &str) -> u64 {
    match kind.into() {
        "B" | "Б" => 2u64.pow(0),
        "KIB" | "КИБ" => 2u64.pow(10),
        "MIB" | "МИБ" => 2u64.pow(20),
        "GIB" | "ГИБ" => 2u64.pow(30),
        "KB" | "КБ" => 10u64.pow(3),
        "MB" | "МБ" => 10u64.pow(6),
        "GB" | "ГБ" => 10u64.pow(9),
        _ => panic!("unknown")
    }
}