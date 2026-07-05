use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-changed=assets/icons/pytask.ico");

    let Some(rc) = find_resource_compiler() else {
        println!(
            "cargo:warning=rc.exe was not found; PyTask.exe will be built without Windows resources"
        );
        return;
    };
    let Some(cvtres) = find_cvtres() else {
        println!(
            "cargo:warning=cvtres.exe was not found; PyTask.exe will be built without Windows resources"
        );
        return;
    };

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR should be set by cargo"));
    let manifest_dir =
        PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR should be set"));
    let version = Version::from_package();
    let manifest_path = out_dir.join("pytask.manifest");
    let rc_path = out_dir.join("pytask.rc");
    let res_path = out_dir.join("pytask.res");
    let resource_obj_path = out_dir.join("pytask_resources.obj");

    fs::write(&manifest_path, windows_manifest_source(&version))
        .unwrap_or_else(|err| panic!("failed to write {}: {err}", manifest_path.display()));
    fs::write(&rc_path, windows_resource_source(&version, &manifest_path))
        .unwrap_or_else(|err| panic!("failed to write {}: {err}", rc_path.display()));

    let status = Command::new(&rc)
        .current_dir(&manifest_dir)
        .arg("/nologo")
        .arg(format!("/fo{}", res_path.display()))
        .arg(&rc_path)
        .status()
        .unwrap_or_else(|err| panic!("failed to run {}: {err}", rc.display()));

    if !status.success() {
        panic!("rc.exe failed while compiling {}", rc_path.display());
    }

    let machine = match env::var("CARGO_CFG_TARGET_ARCH").as_deref() {
        Ok("x86") => "X86",
        Ok("aarch64") => "ARM64",
        _ => "X64",
    };
    let status = Command::new(&cvtres)
        .arg("/NOLOGO")
        .arg(format!("/MACHINE:{machine}"))
        .arg(format!("/OUT:{}", resource_obj_path.display()))
        .arg(&res_path)
        .status()
        .unwrap_or_else(|err| panic!("failed to run {}: {err}", cvtres.display()));

    if !status.success() {
        panic!("cvtres.exe failed while compiling {}", res_path.display());
    }

    println!("cargo:rustc-link-arg={}", resource_obj_path.display());
}

struct Version {
    text: String,
    major: u16,
    minor: u16,
    patch: u16,
    build: u16,
}

impl Version {
    fn from_package() -> Self {
        let text = env::var("CARGO_PKG_VERSION").expect("CARGO_PKG_VERSION should be set");
        let (core, prerelease) = text
            .split_once('-')
            .map_or((text.as_str(), None), |(core, pre)| (core, Some(pre)));
        let mut parts = core.split('.');
        let major = parse_version_part(parts.next(), "major");
        let minor = parse_version_part(parts.next(), "minor");
        let patch = parse_version_part(parts.next(), "patch");
        let build = prerelease
            .and_then(|pre| {
                pre.split('.')
                    .rev()
                    .find_map(|part| part.parse::<u16>().ok())
            })
            .unwrap_or(0);

        Self {
            text,
            major,
            minor,
            patch,
            build,
        }
    }
}

fn parse_version_part(value: Option<&str>, label: &str) -> u16 {
    value
        .unwrap_or_else(|| panic!("missing {label} version component"))
        .parse::<u16>()
        .unwrap_or_else(|err| panic!("invalid {label} version component: {err}"))
}

fn windows_resource_source(version: &Version, manifest_path: &Path) -> String {
    let manifest_path = manifest_path.display().to_string().replace('\\', "\\\\");
    format!(
        r#"#define IDI_PYTASK 1
#define CREATEPROCESS_MANIFEST_RESOURCE_ID 1
#define RT_MANIFEST 24
#define VS_VERSION_INFO 1

IDI_PYTASK ICON "assets\\icons\\pytask.ico"
CREATEPROCESS_MANIFEST_RESOURCE_ID RT_MANIFEST "{manifest_path}"

VS_VERSION_INFO VERSIONINFO
 FILEVERSION {major},{minor},{patch},{build}
 PRODUCTVERSION {major},{minor},{patch},{build}
 FILEFLAGSMASK 0x3fL
 FILEFLAGS 0x0L
 FILEOS 0x40004L
 FILETYPE 0x1L
 FILESUBTYPE 0x0L
BEGIN
    BLOCK "StringFileInfo"
    BEGIN
        BLOCK "040904b0"
        BEGIN
            VALUE "CompanyName", "PyTask\0"
            VALUE "FileDescription", "PyTask Macro Recorder\0"
            VALUE "FileVersion", "{text}\0"
            VALUE "InternalName", "PyTask\0"
            VALUE "OriginalFilename", "PyTask.exe\0"
            VALUE "ProductName", "PyTask\0"
            VALUE "ProductVersion", "{text}\0"
        END
    END
    BLOCK "VarFileInfo"
    BEGIN
        VALUE "Translation", 0x0409, 1200
    END
END
"#,
        major = version.major,
        minor = version.minor,
        patch = version.patch,
        build = version.build,
        text = version.text,
        manifest_path = manifest_path,
    )
}

fn windows_manifest_source(version: &Version) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <assemblyIdentity
    type="win32"
    name="PyTask"
    version="{major}.{minor}.{patch}.{build}"
    processorArchitecture="*" />
  <description>PyTask Macro Recorder</description>
  <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
    <security>
      <requestedPrivileges>
        <requestedExecutionLevel level="asInvoker" uiAccess="false" />
      </requestedPrivileges>
    </security>
  </trustInfo>
  <dependency>
    <dependentAssembly>
      <assemblyIdentity
        type="win32"
        name="Microsoft.Windows.Common-Controls"
        version="6.0.0.0"
        processorArchitecture="*"
        publicKeyToken="6595b64144ccf1df"
        language="*" />
    </dependentAssembly>
  </dependency>
  <compatibility xmlns="urn:schemas-microsoft-com:compatibility.v1">
    <application>
      <supportedOS Id="{{8e0f7a12-bfb3-4fe8-b9a5-48fd50a15a9a}}" />
    </application>
  </compatibility>
  <application xmlns="urn:schemas-microsoft-com:asm.v3">
    <windowsSettings>
      <dpiAwareness xmlns="http://schemas.microsoft.com/SMI/2016/WindowsSettings">PerMonitorV2, PerMonitor</dpiAwareness>
      <dpiAware xmlns="http://schemas.microsoft.com/SMI/2005/WindowsSettings">true/pm</dpiAware>
    </windowsSettings>
  </application>
</assembly>
"#,
        major = version.major,
        minor = version.minor,
        patch = version.patch,
        build = version.build,
    )
}

fn find_resource_compiler() -> Option<PathBuf> {
    env::var_os("RC")
        .map(PathBuf::from)
        .filter(|path| path.is_file())
        .or_else(find_in_path)
        .or_else(find_in_windows_kits)
}

fn find_cvtres() -> Option<PathBuf> {
    env::var_os("CVTRES")
        .map(PathBuf::from)
        .filter(|path| path.is_file())
        .or_else(|| {
            let path_env = env::var_os("PATH")?;
            env::split_paths(&path_env)
                .map(|dir| dir.join("cvtres.exe"))
                .find(|path| path.is_file())
        })
        .or_else(find_in_visual_studio)
}

fn find_in_path() -> Option<PathBuf> {
    let path_env = env::var_os("PATH")?;
    env::split_paths(&path_env)
        .map(|dir| dir.join("rc.exe"))
        .find(|path| path.is_file())
}

fn find_in_windows_kits() -> Option<PathBuf> {
    let program_files = env::var_os("ProgramFiles(x86)")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\Program Files (x86)"));
    let kit_bin = program_files.join("Windows Kits").join("10").join("bin");
    let arch_dir = match env::var("CARGO_CFG_TARGET_ARCH").as_deref() {
        Ok("x86") => "x86",
        Ok("aarch64") => "arm64",
        _ => "x64",
    };

    let mut candidates = read_dirs(&kit_bin)
        .into_iter()
        .map(|dir| dir.join(arch_dir).join("rc.exe"))
        .filter(|path| path.is_file())
        .collect::<Vec<_>>();
    candidates.sort();
    candidates.pop()
}

fn find_in_visual_studio() -> Option<PathBuf> {
    let program_files_x86 = env::var_os("ProgramFiles(x86)")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\Program Files (x86)"));
    let vc_tools = program_files_x86
        .join("Microsoft Visual Studio")
        .join("2022")
        .join("BuildTools")
        .join("VC")
        .join("Tools")
        .join("MSVC");
    let host_arch = "Hostx64";
    let target_arch = match env::var("CARGO_CFG_TARGET_ARCH").as_deref() {
        Ok("x86") => "x86",
        Ok("aarch64") => "arm64",
        _ => "x64",
    };

    let mut candidates = read_dirs(&vc_tools)
        .into_iter()
        .map(|dir| {
            dir.join("bin")
                .join(host_arch)
                .join(target_arch)
                .join("cvtres.exe")
        })
        .filter(|path| path.is_file())
        .collect::<Vec<_>>();
    candidates.sort();
    candidates.pop()
}

fn read_dirs(path: &Path) -> Vec<PathBuf> {
    fs::read_dir(path)
        .ok()
        .into_iter()
        .flat_map(|entries| entries.filter_map(Result::ok))
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .collect()
}
