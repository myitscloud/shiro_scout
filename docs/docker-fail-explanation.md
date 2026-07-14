Step	What happens	Result
1️⃣ Settings Save	TS sends { workspacePath: "C:\\projects" } to Rust	workspace_path silently set to "" 😱
2️⃣ Settings Load	Rust returns { workspace_path: "" } to TS	TS has no workspacePath → undefined
3️⃣ Auto-start in App.tsx:115	Reads settings.workspacePath  ''	'undefined'  '' → empty string
4️⃣ createSandbox call	Sends workspace_path: '' to container.rs	build_host_config sees empty → adds tmpfs instead of bind mount
5️⃣ docker create result	Container has /workspace tmpfs, not C:\projects mount	User sees no files




I've read all 5 layers of the mount pipeline. Here's your answer:

Root Cause: Serde field name mismatch 🔴
TypeScript sends workspacePath (camelCase) to Rust. Rust's settings.rs has:

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppSettings {
    pub workspace_path: String,   // <-- snake_case, expects "workspace_path"
    pub mount_workspace: bool,
    // ...
}
No #[serde(rename_all = "camelCase")] on the struct. So serde looks for "workspace_path" in the JSON, finds "workspacePath" from TypeScript, and silently drops it (because #[serde(default)] ignores unknown fields). workspace_path defaults to "" every time.

The full broken chain:
Step	What happens	Result
1️⃣ Settings Save	TS sends { workspacePath: "C:\\projects" } to Rust	workspace_path silently set to "" 😱
2️⃣ Settings Load	Rust returns { workspace_path: "" } to TS	TS has no workspacePath → undefined
3️⃣ Auto-start in App.tsx:115	Reads settings.workspacePath ''	'undefined' '' → empty string
4️⃣ createSandbox call	Sends workspace_path: '' to container.rs	build_host_config sees empty → adds tmpfs instead of bind mount
5️⃣ docker create result	Container has /workspace tmpfs, not C:\projects mount	User sees no files
How to confirm right now:
Open Settings in the app. See if C:\projects shows or the field is blank. That's the bug in action — the path you set is silently dropped every time you save.

The Fix (2 file patches):
Fix 1: settings.rs — Add rename_all = "camelCase"
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]  // ← ADD THIS
pub struct AppSettings {
Fix 2: Run sandbox as root for testing
In container.rs line ~189:

user: None,  // ← change from Some("1000:1000") to None = root
init: Some(false),  // already false
readonly_rootfs: Some(false),  // already false
cap_drop: None,  // already None (no caps dropped)
security_opt: None,  // already None
Want me to apply both patches right now?

Wait, before you code. Explain to me in details what is going on and what you found. I want to understand this issue so i can avoid it in the future.


Here's the full breakdown of the workspace mount bug and why it happens. I'll walk through the entire data flow so you see the exact line where the data disappears.

1. The 5-Layer Mount Pipeline
For the workspace mount to work, 5 separate files must agree on one thing: what name the folder path has. Here's the complete chain:

Step	Layer	File	What Happens
1	Settings UI	Settings.tsx	You type/pick C:\\projects, it's stored in workspacePath (camelCase)
2	Save	AppContext.tsx	updateSettings merges into state, calls saveSettingsToBackend(newSettings)
3	IPC	tauri-commands.ts	invoke('save_settings', { settings }) sends the JS object as JSON to Rust
4	Rust deserialize	settings.rs	serde_json::from_str tries to match JSON keys to Rust struct fields
5	Rust serialize	settings.rs	When you open settings next time, Rust writes JSON back with its field names
6	Frontend load	AppContext.tsx → App.tsx	loadSettings() gets Rust data, stores in React state
7	Sandbox create	App.tsx:115	Reads settings.workspacePath, passes to createSandbox({ workspace_path })
8	Docker bind	container.rs	build_host_config creates bind mount only if path is non-empty
2. The Exact Breaking Point (Step 4)
Here's the root cause — a naming mismatch between TypeScript and Rust:

TypeScript (tauri-commands.ts) sends this JSON to Rust:

{
  "theme": "dark",
  "workspacePath": "C:\\projects",  // ← camelCase
  "mount_workspace": true
}
Rust (settings.rs) expects this structure:

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]  // ← THIS IS THE KEY ATTRIBUTE
pub struct AppSettings {
    pub theme: String,
    pub workspace_path: String,  // ← snake_case! Expects "workspace_path"
    pub mount_workspace: bool,
}
3. What #[serde(default)] Actually Does
This is the critical Rust behavior you need to understand:

#[serde(default)] on the struct means: "If a JSON key doesn't match any field name, silently ignore it and use the Default value for any unmatched fields"
When serde sees "workspacePath" in JSON, it looks for a field named workspace_path — doesn't find one → it ignores the key entirely
workspace_path is not in the JSON with its exact name → gets String::new() (empty string)
So the JSON {"workspacePath":"C:\\projects"} saves successfully (no error), but workspace_path stays empty.

4. The Domino Effect
Once workspace_path is empty in Rust, the whole chain breaks:

Save: workspacePath: "C:\\projects" → Rust saves "" → written to disk as "workspace_path": ""
Load: Rust reads "workspace_path": "" → sends { workspacePath: "" } back to TypeScript
UI: Settings opens → workspacePath is blank (or "") — your path is gone
Auto-start (App.tsx:115):
const wsPath = settingsRef.current.mount_workspace 
  ? settingsRef.current.workspacePath || ''  // ← empty string!
  : '';
await createSandbox({
  workspace_path: wsPath,  // ← sends "" to container.rs
});
Container creation (container.rs, build_host_config):
if !config.workspace_path.is_empty() {
    binds.push(format!("{}:/workspace:rw", config.workspace_path));
} else {
    // ← This path runs! Adds tmpfs instead of bind mount
    tmpfs_map.insert("/workspace".to_string(), "rw,nosuid,size=512M".to_string());
}
5. Why workspace_path Matched (Other Fields Don't Break)
You might wonder: "Why is mount_workspace spelled the same in both but that works?"

Because it IS spelled the same — Rust has mount_workspace and TypeScript sends mount_workspace already (look at the handleSave function in Settings.tsx). The naming was inconsistent: the TypeScript interface uses workspacePath but sends workspacePath to Rust, while other fields were already using snake_case.

6. The Fix (Two Options)
Option A (Recommended) — Add #[serde(rename_all = "camelCase")] to the Rust struct:

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]  // <-- tells serde: expect camelCase
pub struct AppSettings {
    pub workspace_path: String,  // JSON key becomes "workspacePath"
    pub mount_workspace: bool,   // JSON key becomes "mountWorkspace"
    // ...
}
This tells serde: "This struct was designed for camelCase JSON" — it automatically converts every field name. This is the cleanest fix because it's one line.

Option B — Use #[serde(alias = "workspacePath")] on just that field:

#[serde(alias = "workspacePath")]
pub workspace_path: String,
This accepts both workspace_path AND workspacePath in JSON. More targeted but harder to maintain.

7. Lesson: The TypeScript ↔ Rust Naming Convention Gap
This is a very common bug in Tauri apps because:

Language	Convention	Example
TypeScript/JS	camelCase	workspacePath
Rust	snake_case	workspace_path
Tauri IPC (JSON)	Depends on serde config	#[serde(rename_all = "camelCase")] bridges the gap
Rule of thumb: When crossing the TypeScript ↔ Rust boundary in Tauri, you have two choices:

Send snake_case from TypeScript (already done in parts of this codebase like mount_workspace)
Add #[serde(rename_all = "camelCase")] to the Rust struct
Mixing them (some fields camelCase, others snake_case) always breaks.

Want me to apply the fix now?


