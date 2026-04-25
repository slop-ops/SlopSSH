Muon SSH Rust/Tauri -- Buildability & Release Readiness Report
Executive Summary
The Rust workspace and Svelte frontend each build successfully in isolation. However, the Tauri production build (cargo tauri build) is broken due to path misconfiguration in tauri.conf.json. The CI pipeline's build-linux job will fail for the same reason. Several secondary issues also need resolution before a release.
---
1. What Actually Builds Successfully
Check	Result	Details
cargo build --workspace	PASS	Both muon-core and muon-tauri compile cleanly
cargo test --workspace	PASS	154 tests, 0 failures, 0 ignored
cargo clippy --workspace --all-targets -- -D warnings	PASS	Zero warnings
npm run build (frontend)	PASS	Vite produces dist/ with index.html + assets
npm run check (svelte-check)	PASS	0 errors, 42 a11y warnings only
npm test (vitest)	PASS	14 tests pass in 333ms
cargo tauri build	FAIL	npm cannot find package.json (path issue)
cargo fmt --check	FAIL	97 lines of formatting diff
npm run lint	FAIL	eslint not installed
---
2. Blocking Issues (Must Fix Before Build/Release)
2.1 CRITICAL: tauri.conf.json Path Configuration is Broken
File: /home/pcancer/Documents/muon-ssh-rust/crates/muon-tauri/tauri.conf.json
The Tauri config resolves paths relative to the directory containing tauri.conf.json (crates/muon-tauri/). The current paths are wrong:
"frontendDist": "../dist",           // Resolves to crates/dist -- WRONG
"beforeBuildCommand": "npm run build", // Runs from crates/ -- WRONG
"beforeDevCommand": "npm run dev",    // Runs from crates/ -- WRONG
Actual failure when running cargo tauri build:
npm error enoent Could not read package.json: Error: ENOENT: no such file or directory,
  open '/home/pcancer/Documents/muon-ssh-rust/crates/package.json'
Error beforeBuildCommand `npm run build` failed with exit code 254
Required fixes in tauri.conf.json:
"build": {
    "beforeDevCommand": "npm --prefix ../../frontend run dev",
    "devUrl": "http://localhost:1420",
    "beforeBuildCommand": "npm --prefix ../../frontend run build",
    "frontendDist": "../../frontend/dist"
}
Alternatively, add a "frontendDir" or use cwd patterns specific to Tauri 2 conventions. The npm --prefix approach is the most straightforward fix that preserves the current directory layout.
2.2 CRITICAL: CI Pipeline Will Fail
File: /home/pcancer/Documents/muon-ssh-rust/.github/workflows/ci.yml
The build-linux job runs:
- name: Build Tauri app
  working-directory: crates/muon-tauri
  run: cargo tauri build
This will hit the exact same npm run build failure. The CI also does not install npm ci before the Tauri build step -- it only does npm ci in the test job. The build-linux job would need:
- name: Install frontend dependencies
  working-directory: frontend
  run: npm ci
before the cargo tauri build step, AND the tauri.conf.json paths must be fixed.
2.3 IMPORTANT: cargo fmt --check Fails
There are formatting violations in:
- /home/pcancer/Documents/muon-ssh-rust/crates/muon-core/src/plugin/host.rs (long lines not wrapped)
- /home/pcancer/Documents/muon-ssh-rust/crates/muon-tauri/src/main.rs (long lines not wrapped)
This causes the CI Check formatting step to fail. Run cargo fmt to fix.
---
3. Secondary Issues (Should Fix Before Release)
3.1 ESLint Not Properly Configured
- package.json has "lint": "eslint src/ || true" but eslint is not a devDependency
- No ESLint config file exists (no .eslintrc.*, no eslint.config.*)
- The || true in the script silently masks the failure
- The CI step npm run lint would succeed vacuously
Fix: Either add eslint + config properly, or remove the lint script and CI step.
3.2 tauri.conf.json $schema URL Returns 404
File: /home/pcancer/Documents/muon-ssh-rust/crates/muon-tauri/tauri.conf.json line 2
Current value:
"$schema": "https://raw.githubusercontent.com/nicepage/nicepage.github.io/main/tauri/tauri.conf.schema.json"
This URL returns HTTP 404. For Tauri 2, the correct schema URL would be:
https://raw.githubusercontent.com/tauri-apps/tauri/refs/heads/dev/packages/cli/config.schema.json
This is not a build blocker but causes IDE validation failures and is misleading.
3.3 Bundle Identifier Warning
The identifier com.muon-ssh.app ends with .app, which conflicts with the macOS application bundle extension. Tauri warns:
Warn The bundle identifier "com.muon-ssh.app" set in "tauri.conf.json" identifier ends with ".app".
This is not recommended because it conflicts with the application bundle extension on macOS.
Fix: Change to com.muon-ssh.desktop or com.muon-ssh.client.
3.4 Duplicate #[allow(dead_code)] Attribute
File: /home/pcancer/Documents/muon-ssh-rust/crates/muon-tauri/src/state.rs lines 22-23
#[allow(dead_code)]
#[allow(dead_code)]
pub credential_cache: muon_core::credentials::CredentialCache,
The same attribute is duplicated. This is harmless but sloppy.
3.5 Large Frontend Bundle
The Vite build produces a single 594 KB JS chunk (index-D445AKDm.js), exceeding the 500 KB warning threshold. Consider code-splitting with dynamic imports, especially for tools panels and xterm.js addons.
3.6 withGlobalTauri: false + No @tauri-apps/plugin-shell JS Import
The package.json includes @tauri-apps/plugin-shell as a dependency, and the tauri.conf.json uses tauri_plugin_shell::init() in Rust. However, I found no Svelte files actually importing from @tauri-apps/plugin-shell -- the shell:allow-open permission in capabilities is declared but unused in the frontend. Not a build issue but suggests dead dependency or incomplete feature.
---
4. Stub / Incomplete Implementation Check
Rust code: No todo!(), unimplemented!(), TODO, FIXME, HACK, or STUB markers found anywhere in the Rust codebase. All 67 IPC commands have full implementations.
Frontend code: No TODO, FIXME, HACK, or stub markers found. The only placeholder matches are HTML <input placeholder="..."> attributes, which are normal UI text.
Verdict: The codebase appears to be a complete implementation with no stubs.
---
5. Missing Files or Configurations
Item	Status	Impact
crates/muon-tauri/tauri.conf.json	Present, paths wrong	Blocks production build
crates/muon-tauri/capabilities/default.json	Present, valid	OK
crates/muon-tauri/build.rs	Present	OK
crates/muon-tauri/icons/*	All present (32x32, 128x128, 128x128@2x, icon.icns, icon.ico)	OK
frontend/src/i18n/*.json	All 7 present (en, es, ru, fr, de, pt, cn)	OK
frontend/public/favicon.svg	Present	OK
ESLint config	Missing	CI lint step is a no-op
.github/workflows/ci.yml build-linux	Missing npm ci step	CI build job would fail even with path fix
---
6. Recommended Next Steps (Priority Order)
1. Fix tauri.conf.json paths -- Update frontendDist, beforeBuildCommand, and beforeDevCommand to use ../../frontend/dist and npm --prefix ../../frontend. This is the single highest-priority fix.
2. Run cargo fmt -- Apply formatting to fix the CI Check formatting step.
3. Fix the CI build-linux job -- Add npm ci step for the frontend, and verify the Tauri build works after the path fix.
4. Fix or remove ESLint -- Either add eslint as a devDependency with a proper config file, or remove the lint script and CI step.
5. Fix the $schema URL -- Update to a valid Tauri 2 schema URL.
6. Change bundle identifier -- Avoid the .app suffix.
7. Remove duplicate #[allow(dead_code)] -- Clean up state.rs.
8. Consider code-splitting -- Address the 594 KB JS bundle size warning.
9. Verify cargo tauri build end-to-end -- After fixing paths, do a full release build to confirm deb/AppImage generation works on Linux.
