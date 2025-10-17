from pathlib import Path
import shutil
from dataclasses import dataclass
import re
import os
from functools import lru_cache

import toml
from frozendict import frozendict

type PathLike = os.PathLike | str | Path

DIR = Path(__file__).parent if "__file__" in locals() else Path.cwd()
ident_re_src = r"[A-Za-z_][A-Za-z0-9\-_]*"
ident_re = re.compile(ident_re_src)
export_file_re = re.compile(r"^\/\/\s*->\s+")
public_item_re = re.compile(
    rf"^#let\s*\/\*\s*pub(?:\s+as\s+(?P<alias>{ident_re_src}))?\s*\*\/\s*(?P<name>{ident_re_src})"
)


@dataclass
class PublicItem:
    name: str
    alias: str | None = None


def ensure_typst_project(project_dir: Path):
    toml_file = project_dir / "typst.toml"
    impl_dir = project_dir / "src/_impl"
    if not toml_file.exists() or not toml_file.is_file():
        raise FileNotFoundError(
            "`typst.toml` not found in project directory. Not a valid Typst package."
        )
    if not impl_dir.exists() or not impl_dir.is_dir():
        raise FileNotFoundError(
            "`src/_impl` directory not found in project directory. Not a valid Typst package."
        )


def new_project(project_dir: PathLike, project_name: str | None = None):
    project_dir = Path(project_dir)
    if project_dir.exists():
        raise FileExistsError(
            f"Project directory `{project_dir}` already exists. Cannot create a new project."
        )
    project_dir.mkdir(parents=True)
    if project_name is None:
        project_name = project_dir.name

    # initialize `src` directory
    (project_dir / "src/_impl").mkdir(parents=True)
    (project_dir / "src/lib.typ").write_text("")
    (project_dir / ".gitignore").write_text("")
    (project_dir / "README.orig.md").write_text("# Typst package: {{name}}\n")

    project_info = dict(
        package=dict(
            name=project_name,
            version="0.1.0",
            entrypoint="src/lib.typ",
        )
    )
    toml.dump(project_info, project_dir / "typst.toml")


@lru_cache
def get_package_info(project_dir: os.PathLike):
    project_dir = Path(project_dir)
    return frozendict(toml.load(project_dir / "typst.toml")["package"])


def generate_export(project_dir: os.PathLike):
    project_dir = Path(project_dir)
    ensure_typst_project(project_dir)
    src_dir = project_dir / "src"
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
                f"exporting `{source_file.relative_to(project_dir, walk_up=True).as_posix()}` "
                f"to `{export_file.relative_to(project_dir, walk_up=True).as_posix()}`"
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


def generate_readme(project_dir):
    project_dir = Path(project_dir)
    ensure_typst_project(project_dir)
    package_info = get_package_info(project_dir)

    # replace package info fields in `README.orig.md` and write to `README.md`
    readme_orig_file = project_dir / "README.orig.md"
    readme_file = project_dir / "README.md"
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


def copy_files(project_dir: os.PathLike):
    project_dir = Path(project_dir)
    ensure_typst_project(project_dir)
    package_info = get_package_info(project_dir)
    package_version = package_info["version"]

    # copy files that need to be packed to `packed/<version>`
    target_dir = project_dir / "packed" / package_version
    files_to_copy = (
        project_dir / "src",
        project_dir / "assets",
        project_dir / "README.md",
        project_dir / "LICENSE.txt",
        project_dir / "typst.toml",
    )
    if target_dir.exists():
        shutil.rmtree(target_dir)
    target_dir.mkdir(exist_ok=True, parents=True)
    for file in files_to_copy:
        if file.exists():
            file.copy_into(target_dir)


if __name__ == "__main__":
    # args = sys.argv[1:]
    generate_export(DIR)
    generate_readme(DIR)
    copy_files(DIR)

    # print(f"Package `{package_name}:{package_version}` packed successfully.")
    # print("Now copy it to Typst's `packages` repository and submit a PR.")
