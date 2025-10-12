from pathlib import Path
import shutil

import toml

DIR = Path(__file__).parent if "__file__" in locals() else Path.cwd()

if __name__ == "__main__":
    package_info = toml.load(DIR / "typst.toml")["package"]
    package_name: str = package_info["name"]
    package_version: str = package_info["version"]

    readme_orig_file = DIR / "README.orig.md"
    readme_file = DIR / "README.md"
    readme_src = (
        readme_orig_file.read_text()
        .replace("{{name}}", package_name)
        .replace("{{version}}", package_version)
    )
    readme_file.write_text(readme_src)

    target_dir = DIR / "packed" / package_version
    files_to_copy = (
        DIR / "src",
        DIR / "README.md",
        DIR / "LICENSE.txt",
        DIR / "typst.toml",
    )
    if target_dir.exists():
        shutil.rmtree(target_dir)
    target_dir.mkdir(exist_ok=True, parents=True)
    for file in files_to_copy:
        file.copy_into(target_dir)

    print(f"Package {package_name} {package_version} packed successfully.")
    print("Now copy it to Typst's `packages` repository and submit a PR.")
