# Project Aegis — User Personas (v1.0)

> **Date:** 2026-07-08
> **Source:** Expanded from Merge PRD §3
> **Intended for:** UI/UX design, documentation tone, feature prioritization, and acceptance criteria

---

## Persona 1: Alex Chen — Senior DevOps Engineer

| Attribute | Detail |
|-----------|--------|
| **Title** | Senior DevOps Engineer |
| **Age** | 34 |
| **Company** | Mid-size SaaS company (500 employees) |
| **Location** | Remote, based in Austin, TX |
| **Technical Level** | Expert — k8s, Terraform, CI/CD, scripting |
| **Operating System** | Windows 11 (primary) + Ubuntu servers |

### Goals
- Autonomously debug production logs and error stacks without spinning up a full dev environment
- Generate Terraform patches and test them in isolation before applying to prod
- Write and test shell scripts for incident response automation
- Quickly analyze Grafana/CloudWatch metrics exported as JSON

### Pain Points
- Setting up Docker + Python environments for one-off debugging takes 15+ minutes
- Afraid of AI hallucinating destructive commands (`rm -rf`, `kubectl delete`)
- Existing AI coding tools (Cursor, Copilot) don't isolate execution from his host machine
- Needs HITL checkpoints with timeout so he can review dangerous operations

### Success Scenario
Alex double-clicks Aegis, types "/analyze /logs/error.log and suggest a fix", the agent spawns a sandbox, pulls the log file, runs grep/sed/awk analysis, finds the root cause (a missing env var), writes a Terraform patch, tests it in-container, and presents the diff for approval — all without ever leaving his desktop.

### Environment
Docker Desktop for Windows, Ollama running llama3.3-70b locally, 32GB RAM workstation. Needs air-gapped sandbox because production logs contain sensitive data.

---

## Persona 2: Maya Patel — Data Analyst

| Attribute | Detail |
|-----------|--------|
| **Title** | Senior Data Analyst |
| **Age** | 29 |
| **Company** | Fintech startup (50 employees) |
| **Location** | San Francisco, CA (in-office hybrid) |
| **Technical Level** | Intermediate — Python, SQL, Pandas, matplotlib |
| **Operating System** | macOS (M3 MacBook Pro) |

### Goals
- Generate and execute Python scripts for dataset processing without setting up venvs or managing packages
- Create visual reports (charts, graphs) from CSV/JSON exports directly in the chat
- Automate repetitive data cleanup tasks (missing values, outliers, normalization)
- Export results as formatted tables, PNG charts, or PDF reports

### Pain Points
- Switching between Jupyter, VS Code, and terminal to run one-off analyses is inefficient
- Package dependency hell when a script needs `pandas 2.x` but another tool needs `pandas 1.x`
- No way to generate charts without opening a full Python IDE
- Wants to share results as clean reports, not terminal output or notebook JSON

### Success Scenario
Maya drags a 500MB CSV export into Aegis and types "Clean this dataset (remove duplicates, fill missing values, normalize amounts). Then create a bar chart of monthly revenue by region." The agent writes a pandas script, executes it in the sandbox, prints the summary statistics, renders a matplotlib chart, and displays it inline in the chat.

### Environment
macOS with Orbstack for Docker, prefers cloud LLM (Claude Sonnet via API), needs VNC desktop for matplotlib rendering. Works from a coffee shop with intermittent WiFi.

---

## Persona 3: Jamie Rodriguez — Hobbyist Developer

| Attribute | Detail |
|-----------|--------|
| **Title** | Hobbyist Developer / CS Student |
| **Age** | 22 |
| **Company** | University (senior year) + freelance gigs |
| **Location** | Chicago, IL (campus housing) |
| **Technical Level** | Intermediate — React, Node.js, Python basics |
| **Operating System** | Windows 11 (gaming laptop) |

### Goals
- Explore AI coding agents without spending hours on setup, Docker config, or API key management
- Use Aegis as a learning tool — understand how agents work by observing their reasoning
- Build small personal projects (web apps, Discord bots, CLI tools) with AI assistance
- Run everything locally to avoid API costs (student budget)

### Pain Points
- Intimidated by terminal-based AI tools (needs GUI)
- Doesn't have a credit card for cloud API keys
- Docker Desktop eats his laptop's 16GB RAM if not configured properly
- Wants a "one-click start" experience — no config files, no command line

### Success Scenario
Jamie downloads Aegis from the website, double-clicks the installer, the first-run wizard detects Docker Desktop, pulls the agent image, detects Ollama (already installed), and presents a chat interface — all within 2 minutes. He types "Help me build a React to-do app with local storage" and watches the agent build it step by step.

### Environment
Windows 11 gaming laptop (RTX 3060, 16GB RAM), Docker Desktop with WSL2 backend, Ollama with Llama 3.2 3B (fits in VRAM). Budget: $0/month for AI — fully local.

---

## Persona 4: Dr. Sarah Okonkwo — Research Scientist

| Attribute | Detail |
|-----------|--------|
| **Title** | Research Scientist (Computational Biology) |
| **Age** | 41 |
| **Company** | Academic research lab at a major university |
| **Location** | Boston, MA |
| **Technical Level** | Advanced — Python, R, Bash, some C++ |
| **Operating System** | Fedora Linux (workstation) + macOS (laptop) |

### Goals
- Automate literature review: scrape PubMed/arXiv, aggregate findings, synthesize into structured documents
- Run headless browser scraping of scientific databases and data portals
- Generate reproducible analysis pipelines with version-controlled outputs
- Collaborate with colleagues by sharing Aegis-generated analysis reports

### Pain Points
- Needs to scrape dynamic web content (JavaScript-rendered pages) — static curl/wget won't work
- Research ethics require all data processing to happen locally (no cloud LLM for sensitive data)
- Papers are behind logins — needs browser automation with session cookies
- Outputs must be reproducible and timestamped for publication integrity

### Success Scenario
Sarah pastes 5 PubMed PMIDs into Aegis and types "Scrape the abstracts, find papers about CRISPR off-target effects published after 2024, extract key findings into a table, and summarize the top 3 challenges." The agent runs Playwright headless browser in the sandbox, navigates PubMed, fetches full abstracts, runs NLP extraction, and returns a structured markdown report.

### Environment
Fedora Linux 40, 64GB RAM, 16-core Threadripper. Uses Ollama with Mixtral 8x7B for local inference. Needs air-gapped sandbox for HIPAA-adjacent research data.

---

## Persona 5: Taylor Wu — IT Support Lead

| Attribute | Detail |
|-----------|--------|
| **Title** | IT Support Team Lead |
| **Age** | 38 |
| **Company** | Enterprise (10,000+ employees) |
| **Location** | New York, NY (office) |
| **Technical Level** | Intermediate — PowerShell, Windows admin, basic scripting |
| **Operating System** | Windows 11 (enterprise-managed) |

### Goals
- Automate repetitive IT support tasks: reset passwords, check AD status, restart services
- Generate PowerShelp scripts for endpoint management and test them before deployment
- Document troubleshooting steps for the L1 helpdesk team
- Access enterprise tools (AD, SCCM, Intune) through a secure, auditable interface

### Pain Points
- Enterprise security policies forbid installing unapproved software — needs IT approval for every tool
- PowerShell scripts that work on his machine may fail on managed endpoints
- Needs audit trail of every action the AI takes (for compliance)
- Cannot use cloud LLMs — enterprise data must stay on-premises

### Success Scenario
Taylor types "Check if the spooler service is running on HOST01, HOST02, and HOST03. If not, restart it and log the action." Aegis runs PowerShell scripts via WinRM through the sandbox, checks each host, restarts the spooler on HOST02, and logs the entire session to an immutable audit trail.

### Environment
Windows 11 Enterprise (locked down), Docker Desktop via IT exception, local Ollama server on-prem. Requires MSI deployment via Intune and Active Directory authentication.

---

## Persona Summary Matrix

| Feature Priority | DevOps Alex | Data Maya | Hobbyist Jamie | Research Sarah | IT Lead Taylor |
|-----------------|-------------|-----------|----------------|----------------|----------------|
| Docker sandbox isolation | ✅ Critical | ✅ Important | ✅ Nice to have | ✅ Critical | ✅ Critical |
| HITL checkpoints | ✅ Critical | 🔲 Low | 🔲 Low | ✅ Important | ✅ Critical |
| Local LLM support | ✅ Important | 🔲 Optional | ✅ Critical | ✅ Critical | ✅ Critical |
| Cloud LLM support | ✅ Important | ✅ Critical | 🔲 Low | 🔲 Low | 🔲 Forbidden |
| Headless browser | 🔲 Low | 🔲 Low | 🔲 Low | ✅ Critical | 🔲 Low |
| VNC desktop streaming | 🔲 Low | ✅ Important | 🔲 Low | ✅ Important | 🔲 Low |
| Code artifact editing | ✅ Important | ✅ Important | ✅ Important | ✅ Important | ✅ Important |
| Audit trail | 🔲 Optional | 🔲 Low | 🔲 Low | ✅ Critical | ✅ Critical |
| First-run wizard | 🔲 Low | 🔲 Low | ✅ Critical | 🔲 Low | 🔲 Low |
| MSI/Enterprise packaging | 🔲 Low | 🔲 Low | 🔲 Low | 🔲 Low | ✅ Critical |
| Offline mode | 🔲 Optional | 🔲 Low | 🔲 Low | ✅ Important | ✅ Important |

---

*Personas are living documents. Update as user research provides real-world data.*
