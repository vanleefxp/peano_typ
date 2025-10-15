from pathlib import Path
import shutil
from dataclasses import dataclass
import re

import toml

DIR = Path(__file__).parent if "__file__" in locals() else Path.cwd()
export_file_re = re.compile(r"^\/\/\s*->\s+")
public_item_re = re.compile(
    r"^#let\s*\/\*\s*pub(?:\s+as\s+(?P<alias>[A-Za-z_][A-Za-z0-9\-_]*))?\s*\*\/\s*(?P<name>[A-Za-z_][A-Za-z0-9\-_]*)"
)


@dataclass
class PublicItem:
    name: str
    alias: str | None = None


if __name__ == "__main__":
    src_dir = DIR / "src"
    impl_dir = src_dir / "_impl"
    # generate export files for each implementation file that have public items
    for source_file in impl_dir.glob("**/*.typ"):
        with source_file.open(encoding="utf-8") as f:
            line = f.readline()
            if (match := export_file_re.match(line)) is not None:
                export_file = src_dir / (line[match.end() :].strip())
            else:
                export_file = src_dir / (source_file.relative_to(impl_dir))

            export_dir = export_file.parent
            export_dir.mkdir(exist_ok=True, parents=True)

            module_doc = ""
            while True:
                line = f.readline()
                if not line.startswith("///"):
                    break
                module_doc += line
            module_doc = module_doc.strip()

            public_items = []
            while True:
                match = public_item_re.match(line)
                if match is not None:
                    public_items.append(
                        PublicItem(name=match.group("name"), alias=match.group("alias"))
                    )
                line = f.readline()
                if len(line) == 0:
                    break

            if len(public_items) == 0:
                continue

            source_file_relative = source_file.relative_to(export_dir, walk_up=True)
            print(
                f"exporting `{source_file.relative_to(DIR, walk_up=True).as_posix()}` "
                f"to `{export_file.relative_to(DIR, walk_up=True).as_posix()}`"
            )
            with export_file.open("w", encoding="utf-8") as f:
                if len(module_doc) > 0:
                    f.write(module_doc)
                    f.write("\n\n")

                f.write(f'#import "{source_file_relative.as_posix()}": (\n')
                for public_item in public_items:
                    match public_item:
                        case PublicItem(name=name, alias=None):
                            f.write(f"  {name},\n")
                        case PublicItem(name=name, alias=alias):
                            f.write(f"  {name} as {alias},\n")
                f.write(")\n\n")
                f.write(
                    "// This is a program-generated file. Do not edit it directly.\n"
                )

    # read package info from `typst.toml`
    package_info = toml.load(DIR / "typst.toml")["package"]
    package_name = package_info["name"]
    package_version = package_info["version"]

    # replace package info fields in `README.orig.md` and write to `README.md`
    readme_orig_file = DIR / "README.orig.md"
    readme_file = DIR / "README.md"
    with (
        readme_orig_file.open(encoding="utf-8") as f1,
        readme_file.open("w", encoding="utf-8") as f2,
    ):
        f2.write(
            "<!-- This is a program-generated file. Do not edit it directly. -->\n\n"
        )
        while len(line := f1.readline()) > 0:
            replaced_line = line
            for k, v in package_info.items():
                if not isinstance(v, str):
                    continue
                replaced_line = replaced_line.replace(f"{{{{{k}}}}}", v)
            f2.write(replaced_line)

    # copy files that need to be packed to `packed/<version>`
    target_dir = DIR / "packed" / package_version
    files_to_copy = (
        DIR / "src",
        DIR / "assets",
        DIR / "README.md",
        DIR / "LICENSE.txt",
        DIR / "typst.toml",
    )
    if target_dir.exists():
        shutil.rmtree(target_dir)
    target_dir.mkdir(exist_ok=True, parents=True)
    for file in files_to_copy:
        if file.exists():
            file.copy_into(target_dir)

    print(f"Package `{package_name}:{package_version}` packed successfully.")
    print("Now copy it to Typst's `packages` repository and submit a PR.")
