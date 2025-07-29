import os
import matplotlib.pyplot as plt
from datetime import datetime

class TxtFile:
    def __init__(self, time: float, name: str):
        self.time = time
        self.name = name
    def date(self) -> str:
        return datetime.fromtimestamp(self.time).strftime('%d-%m-%Y at %H:%M:%S')
    def to_jpg(self) -> str:
        return self.name.removesuffix("txt") + "jpg"

class FileHandler:
    def __init__(self):
        self.files: list[TxtFile] = []
    def grab_text_files(self, dir) -> bool:
        for filename in os.listdir(dir):
            if filename.endswith("txt"):
                time       = os.path.getmtime(filename)
                timed_file = TxtFile(time, filename)
                self.files.append(timed_file)
        return len(self.files) > 0
        
    def filter_by_keyword(self, keyword: str) -> bool:
        if not self.files:
            return False
        filtered_files = []
        for files in self.files:
            if files.name.find(keyword) != -1:
                filtered_files.append(files)
        self.files    = filtered_files
        return len(self.files) > 0
        
    def get_latest(self) -> TxtFile:
        if not self.files:
            return None
        self.files.sort(key= lambda item: item.time, reverse=True)
        return self.files[0]
         
def get_temp_mag(latest_file: TxtFile, separator: str) -> tuple[list, list]:
    temps = []
    mags  = []
    with open(latest_file.name, "r") as f:
        for lines in f.readlines():
            temp_vs_mag = lines.split(separator)
            temps.append(float(temp_vs_mag[0]))
            mags.append(float(temp_vs_mag[1]))
    return (temps, mags)

if __name__ == "__main__":

    file_handler = FileHandler()
    file_handler.grab_text_files(os.getcwd())

    keyword      = "magnetization"    
    if not file_handler.filter_by_keyword(keyword):
        raise RuntimeError(f">> No files containing \"{keyword}\" found! Exiting... ")
    result_file = file_handler.get_latest()
    
    print(f">> Last modified file with word \"{keyword}\":")
    print(f">> - \"{result_file.name}\"")
    print(f">> - Last modifed: {result_file.date()}")

    (temps, mags) = get_temp_mag(result_file, ", ")
    print(">> loading values...")
    if temps and mags:
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
    img_file = result_file.to_jpg()
    plt.savefig(img_file)
    
    print(f">> Plot saved as \"{img_file}\"")