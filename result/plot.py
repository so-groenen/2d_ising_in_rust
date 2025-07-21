import os
import matplotlib.pyplot as plt
from datetime import datetime

class File:
    def __init__(self, time: float, name: str):
        self.time = time
        self.name = name
        
def get_timed_text_files(dir: str) -> list[str]:
    timed_files = []
    for filename in os.listdir(dir):
        if filename.endswith("txt"):
            time       = os.path.getmtime(filename)
            timed_file = File(time, filename)
            timed_files.append(timed_file)
    return timed_files

def filter_by_keyword(timed_files: list[File], keyword: str) -> list[File]:
    filtered_files = []
    for files in timed_files:
        if files.name.find(keyword) != -1:
            filtered_files.append(files)
    return filtered_files

def get_latest_text_file(timed_files: list[File]) -> File:
    timed_files.sort(key=lambda item: item.time, reverse=True)
    last_modified_timed_file = timed_files[0]
    return last_modified_timed_file

def get_temp_mag(latest_file: str, separator: str) -> tuple[list, list]:
    temps = []
    mags  = []
    with open(latest_file, "r") as f:
        for lines in f.readlines():
            temp_vs_mag = lines.split(separator)
            temps.append(float(temp_vs_mag[0]))
            mags.append(float(temp_vs_mag[1]))
    return (temps, mags)

if __name__ == "__main__":

    keyword = "magnetization"
    
    text_files     = get_timed_text_files(os.getcwd())
    filtered_files = filter_by_keyword(text_files, keyword)
    if not len(filtered_files):
        raise RuntimeError(f">> No files containing \"{keyword}\" found! Exiting... ")

    latest_file = get_latest_text_file(text_files)
    date        = datetime.fromtimestamp(latest_file.time).strftime('%d-%m-%Y at %H:%M:%S')
    
    print(f">> Last modified file with word \"{keyword}\":")
    print(f">> - \"{latest_file.name}\"")
    print(f">> - Last modifed: {date}")

    separator = ", "
    (temps, mags) = get_temp_mag(latest_file.name, separator)
    print(">> loading values...")
    if len(temps) and len(mags):
        print(">> Values loaded!")
    else:
        raise RuntimeError(">> Error loading values, check the separator maybe? Exiting...")
    

    plt.figure(figsize=(10,6))
    plt.title("2D Ising: Magnetization vs temperature", fontsize=15)
    plt.scatter(temps, mags, s=15, c='b', marker='x', label="Metropolis algorithm in Rust")
    plt.xlabel("Temperature $[J/k_B]$", fontsize=15)
    plt.ylabel("Average magnetization", fontsize=15)
    plt.xticks(fontsize=14)
    plt.yticks(fontsize=14)
    plt.legend(fontsize=12)
    img_file = latest_file.name.removesuffix("txt") + "jpg"
    plt.savefig(img_file)
    print(f">> Plot saved as \"{img_file}\"")