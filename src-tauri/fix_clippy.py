#!/usr/bin/env python3
"""Batch fix all clippy errors across the shiro-scout codebase."""
import os

base = r"C:\Users\wayne\agent-zero\Shiro-Scout\usr\projects\shiro_scout\src-tauri\src"

def fix_hitl():
    path = os.path.join(base, "hitl.rs")
    with open(path, "r", encoding="utf-8") as f:
        text = f.read()
    
    # Fix format!("{}", hash) → hash.to_string()
    text = text.replace('format!("{}", hash)', 'hash.to_string()')
    
    # Fix all format!("literal") → "literal".to_string()
    fixes = [
        ('format!("Internal error: failed to acquire session lock")',
         '"Internal error: failed to acquire session lock".to_string()'),
        ('format!("Session not found or already expired")',
         '"Session not found or already expired".to_string()'),
        ('format!("Nonce mismatch: session verification failed")',
         '"Nonce mismatch: session verification failed".to_string()'),
        ('format!("Session has timed out")',
         '"Session has timed out".to_string()'),
        ('format!("Failed to emit confirmation event")',
         '"Failed to emit confirmation event".to_string()'),
        ('format!("Internal serialization error")',
         '"Internal serialization error".to_string()'),
        ('format!("Invalid session ID format")',
         '"Invalid session ID format".to_string()'),
    ]
    for old, new in fixes:
        text = text.replace(old, new)
    
    # Fix redundant pattern matching: if let Err(_) = ... → if ... .is_err()
    text = text.replace(
        'if let Err(_) = app_handle.emit(event_name, &event)',
        'if app_handle.emit(event_name, &event).is_err()'
    )
    
    # Fix empty line after doc comment (lines 5-6: blank line between doc comment and code)
    with open(path, "r", encoding="utf-8") as f:
        lines = f.readlines()
    # The doc comment for generate_nonce function - find blank line after /// line
    # Actually empty_line_after_doc_comments is about doc comments on imports
    # Let's check more carefully... the error was not listed for hitl.rs for empty_line_after_doc_comments
    
    print(f"hitl.rs: {text.count('format!(')} format! patterns remaining")
    with open(path, "w", encoding="utf-8") as f:
        f.write(text)


def fix_prompts():
    path = os.path.join(base, "prompts", "mod.rs")
    with open(path, "r", encoding="utf-8") as f:
        text = f.read()
    
    # Fix format!("literal") → "literal".to_string()
    text = text.replace(
        'Some(format!("You are an AI assistant. Respond to the user\'s requests."))',
        'Some("You are an AI assistant. Respond to the user\'s requests.".to_string())'
    )
    text = text.replace(
        'Some(format!("You have access to various tools. Use them when appropriate."))',
        'Some("You have access to various tools. Use them when appropriate.".to_string())'
    )
    # Fix empty_line_after_doc_comments (line 3-4 pattern)
    # The doc comment for the module has blank line between /// and `use`
    lines = text.split('\n')
    # Line 2 (0-indexed) is empty line between doc comment and use statement
    # Remove the empty line after doc comment on line 3-4
    # Actually need to check: lines 1-3 are doc comments, line 4 is empty, line 5 is "use"
    # clippy says the last /// line (line 3) has an empty line after it before `use`
    
    print(f"prompts/mod.rs format! count: {text.count('format!(')}")
    with open(path, "w", encoding="utf-8") as f:
        f.write(text)

def fix_sandbox():
    path = os.path.join(base, "sandbox", "mod.rs")
    with open(path, "r", encoding="utf-8") as f:
        lines = f.readlines()
    # Lines 1-7 are doc comment with ///, line 8 is blank, line 9 is `use camino::...`
    # clippy wants: last /// line → no blank line → use statement
    # Remove line 8 (blank line between doc comment and code)
    # line 7 is the last /// line (Tier 3), line 8 is empty, line 9 is use
    if len(lines) >= 9 and lines[7].strip() == "" and lines[8].startswith("use"):
        del lines[7]
        print(f"sandbox/mod.rs: removed blank line after doc comment")
    else:
        print(f"sandbox/mod.rs: line 8='{repr(lines[7])}' line 9='{repr(lines[8])}'")
        print(f"sandbox/mod.rs: no change needed to doc comment spacing")
    with open(path, "w", encoding="utf-8") as f:
        f.writelines(lines)

def fix_agent_mod():
    path = os.path.join(base, "agent", "mod.rs")
    with open(path, "r", encoding="utf-8") as f:
        lines = f.readlines()
    # Line 5 is doc comment, line 6 is empty, line 7 is `pub mod agent`
    # Remove blank line between last /// and the next code
    if len(lines) >= 7 and lines[5].strip() == "" and lines[6].startswith("pub mod"):
        del lines[5]
        print(f"agent/mod.rs: removed blank line after doc comment")
    else:
        print(f"agent/mod.rs: lines 5={repr(lines[4])} 6={repr(lines[5])} 7={repr(lines[6])}")
        print(f"agent/mod.rs: manual check needed")
    with open(path, "w", encoding="utf-8") as f:
        f.writelines(lines)

def fix_env_mod():
    path = os.path.join(base, "env", "mod.rs")
    with open(path, "r", encoding="utf-8") as f:
        lines = f.readlines()
    # Line 5 is the last ///, line 6 is empty, line 7 is `use serde::Serialize`
    if len(lines) >= 7 and lines[5].strip() == "" and lines[6].startswith("use"):
        del lines[5]
        print(f"env/mod.rs: removed blank line after doc comment")
    else:
        print(f"env/mod.rs: lines 5={repr(lines[4])} 6={repr(lines[5])} 7={repr(lines[6])}")
    with open(path, "w", encoding="utf-8") as f:
        f.writelines(lines)

def fix_error():
    path = os.path.join(base, "error.rs")
    with open(path, "r", encoding="utf-8") as f:
        lines = f.readlines()
    # Line 2 is doc comment, line 3 is empty, line 4 is `use serde::Serialize`
    if len(lines) >= 4 and lines[2].strip() == "" and lines[3].startswith("use"):
        del lines[2]
        print(f"error.rs: removed blank line after doc comment")
    else:
        print(f"error.rs: lines 2={repr(lines[1])} 3={repr(lines[2])} 4={repr(lines[3])}")
    with open(path, "w", encoding="utf-8") as f:
        f.writelines(lines)

def fix_credential_manager():
    path = os.path.join(base, "llm", "credential_manager.rs")
    with open(path, "r", encoding="utf-8") as f:
        text = f.read()
    
    # Fix empty_line_after_doc_comments
    lines = text.split('\n')
    # Line 5 is the last ///, line 6 is empty, line 7 is `use serde::Serialize`
    if len(lines) >= 7 and lines[5].strip() == "" and "use serde" in lines[6]:
        del lines[5]
        print(f"credential_manager.rs: removed blank line after doc comment")
    else:
        print(f"credential_manager.rs: line 5/6/7: {repr(lines[4])} {repr(lines[5])} {repr(lines[6])}")
    
    text = '\n'.join(lines)
    
    # Fix field_reassign_with_default: replace CREDENTIALW::default() + field assignments
    # with proper struct literal init
    old_init = '''        let mut cred = CREDENTIALW::default();
        cred.Type = windows::Win32::Security::Credentials::CRED_TYPE(CRED_TYPE_GENERIC);
        cred.TargetName = PWSTR::from_raw(target_wide.as_ptr() as *mut u16);
        cred.CredentialBlobSize = key_bytes.len() as u32;
        cred.CredentialBlob = key_bytes.as_ptr() as *mut u8;
        cred.Persist = windows::Win32::Security::Credentials::CRED_PERSIST(CRED_PERSIST_LOCAL_MACHINE);
        // UserName is optional for generic credentials
        cred.UserName = PWSTR::from_raw(std::ptr::null_mut());'''
    
    new_init = '''        let cred = CREDENTIALW {
            Type: windows::Win32::Security::Credentials::CRED_TYPE(CRED_TYPE_GENERIC),
            TargetName: PWSTR::from_raw(target_wide.as_ptr() as *mut u16),
            CredentialBlobSize: key_bytes.len() as u32,
            CredentialBlob: key_bytes.as_ptr() as *mut u8,
            Persist: windows::Win32::Security::Credentials::CRED_PERSIST(CRED_PERSIST_LOCAL_MACHINE),
            UserName: PWSTR::from_raw(std::ptr::null_mut()),
            ..Default::default()
        };'''
    
    if old_init in text:
        text = text.replace(old_init, new_init)
        print(f"credential_manager.rs: fixed field_reassign_with_default")
    else:
        print(f"credential_manager.rs: field_reassign pattern not found - might already be fixed")
    
    # Fix new_without_default: add Default impl for MockCredentialManager
    if "impl Default for MockCredentialManager" not in text:
        # Find the location just after MockCredentialManager's new() method
        # Insert after line `    }` closing `pub fn new()`
        text = text.replace(
            "impl MockCredentialManager {\n    pub fn new() -> Self {\n        Self {\n            store: HashMap::new(),\n        }\n    }",
            "impl MockCredentialManager {\n    pub fn new() -> Self {\n        Self {\n            store: HashMap::new(),\n        }\n    }\n\nimpl Default for MockCredentialManager {\n    fn default() -> Self {\n        Self::new()\n    }\n}"
        )
        print(f"credential_manager.rs: added Default impl for MockCredentialManager")
    
    # Fix new_without_default for PromptStore (this is in prompts/mod.rs, but let me handle it)
    
    print(f"credential_manager.rs: format! count: {text.count('format!(')}")
    with open(path, "w", encoding="utf-8") as f:
        f.write(text)

def fix_tools():
    path = os.path.join(base, "tools", "mod.rs")
    with open(path, "r", encoding="utf-8") as f:
        text = f.read()
    
    # Add Default impl for ToolRegistry
    if "impl Default for ToolRegistry" not in text:
        text = text.replace(
            "impl ToolRegistry {\n    pub fn new() -> Self {",
            "impl Default for ToolRegistry {\n    fn default() -> Self {\n        Self::new()\n    }\n}\n\nimpl ToolRegistry {\n    pub fn new() -> Self {"
        )
        print(f"tools/mod.rs: added Default impl for ToolRegistry")
    else:
        print(f"tools/mod.rs: Default impl already exists")
    
    with open(path, "w", encoding="utf-8") as f:
        f.write(text)

def fix_llm_mod():
    path = os.path.join(base, "llm", "mod.rs")
    with open(path, "r", encoding="utf-8") as f:
        text = f.read()
    
    # Fix while_let_loop: replace `loop { let line_end = match buffer.find('\\n') { Some(pos) => pos, None => break, };`
    # with `while let Some(pos) = buffer.find('\\n') { let line_end = pos;`
    old_loop = '''        loop {
            // Find the next line boundary
            let line_end = match buffer.find('\\n') {
                Some(pos) => pos,
                None => break,
            };'''
    new_loop = '''        while let Some(line_end) = buffer.find('\\n') {'''
    if old_loop in text:
        text = text.replace(old_loop, new_loop)
        print(f"llm/mod.rs: fixed while_let_loop")
    else:
        print(f"llm/mod.rs: loop pattern not found")
    
    with open(path, "w", encoding="utf-8") as f:
        f.write(text)

def fix_health_check():
    path = os.path.join(base, "llm", "health_check.rs")
    with open(path, "r", encoding="utf-8") as f:
        text = f.read()
    
    # Fix if_same_then_else: both branches return same type
    old_block = '''            } else if e.is_connect() {
                HealthError::ConnectionFailed(e.to_string())
            } else {
                HealthError::ConnectionFailed(e.to_string())
            }'''
    new_block = '''            } else {
                HealthError::ConnectionFailed(e.to_string())
            }'''
    if old_block in text:
        text = text.replace(old_block, new_block)
        print(f"health_check.rs: fixed if_same_then_else")
    else:
        print(f"health_check.rs: pattern not found")
        # Try alternative: the first branch might be HealthError::Timeout and then both else-if and else are same
    
    with open(path, "w", encoding="utf-8") as f:
        f.write(text)

def fix_container():
    path = os.path.join(base, "container.rs")
    with open(path, "r", encoding="utf-8") as f:
        text = f.read()
    
    # Fix derivable_impls: replace manual Default impl with #[default] attribute
    old_default = '''impl Default for NetworkMode {
    fn default() -> Self {
        Self::Bridge
    }
}'''
    new_default = '#[default]'
    if old_default in text:
        # Add #[default] to Bridge variant
        text = text.replace(
            "    #[serde(rename = \"bridge\")]\n    Bridge,",
            "    #[serde(rename = \"bridge\")]\n    #[default]\n    Bridge,"
        )
        # Remove manual impl
        text = text.replace(old_default, "")
        # Add #[derive(Default)] after the existing derive
        text = text.replace(
            "#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]\npub enum NetworkMode",
            "#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]\npub enum NetworkMode"
        )
        print(f"container.rs: fixed derivable_impls")
    else:
        print(f"container.rs: manual Default impl not found")
    
    with open(path, "w", encoding="utf-8") as f:
        f.write(text)

def fix_prompts_new_without_default():
    path = os.path.join(base, "prompts", "mod.rs")
    with open(path, "r", encoding="utf-8") as f:
        text = f.read()
    
    if "impl Default for PromptStore" not in text:
        # Insert after closing } of PromptStore
        text = text.replace(
            "impl PromptStore {\n    pub fn new() -> Self {",
            "impl Default for PromptStore {\n    fn default() -> Self {\n        Self::new()\n    }\n}\n\nimpl PromptStore {\n    pub fn new() -> Self {"
        )
        print(f"prompts/mod.rs: added Default impl for PromptStore")
    
    # Also fix empty_line_after_doc_comments (the module doc comment)
    lines = text.split('\n')
    # Line 2: empty line between doc comment and `use`
    if len(lines) >= 4 and lines[2].strip() == "" and "use" in lines[3]:
        del lines[2]
        text = '\n'.join(lines)
        print(f"prompts/mod.rs: removed blank line after doc comment")
    
    with open(path, "w", encoding="utf-8") as f:
        f.write(text)

if __name__ == "__main__":
    fix_hitl()
    fix_prompts()
    fix_prompts_new_without_default()
    fix_sandbox()
    fix_agent_mod()
    fix_env_mod()
    fix_error()
    fix_credential_manager()
    fix_tools()
    fix_llm_mod()
    fix_health_check()
    fix_container()
    print("\nAll fixes applied.")
