<div align="center">
  <pre>
    </br>
                              :GG~        .?Y.                                
    ....        ..      ..   .....      . ^BG: ..       .....                 
 .7555YY7JP^   ~PJ     ~PJ  ?YY5PP~    7YY5BGYYYYJ.   J555YY557.              
.5B?.  :JBB~   !#5     !#5  ...PB~     ...^BG:....    ~:.   .7#5           :^^
7#5     .GB~   !B5     !B5     PB~        :BG.        .~7??J?JBG:      .~JPPPY
?#Y      PB~   !B5     !B5     PB~        :BG.       7GP7~^^^!BG:     ~5GY!:. 
^GB~    7BB~   ^BG.   .YB5     5#7        :BB:       P#!     JBG:    ^GG7     
 ^5G5JJYJPB~    JBP???YYB5     ^5GYJJ?.    7GPJ???.  ~PGJ77?5J5B!    JG5      
   .^~^..GB:     :~!!~. ^^       :~~~~      .^~~~~    .^!!!~. .^:    JG5      
 .?!^^^!5G7                                                          YB5      
 .!?JJJ?!:                                                           75?      
  </br>
Beautiful and cozy terminal-based Git client
 for fun, productivity, and pure Rust joy.  
  </pre>
</div>


![untitled(1)](https://github.com/user-attachments/assets/e39b0ebb-52dc-45bd-a430-0f592d5fc315)

---

## 🌟 Motivation

I am building **guita╭** as a personal exercise in procrustination.

The goal is simple:

- Render Git commit graphs beautifully and efficiently in the terminal.  
- Traverse massive repositories instantly – hundreds of thousands of commits spanning decades.  
- Jump anywhere in history without lag.  
- Enjoy a cozy experience with pleasing colors and smooth, terminal-friendly navigation.  
- Include a fast, built-in diff viewer.  

**Why guita╭?** Because working with Git in the terminal should feel fun, not frustrating.

---

## ⚠️ Work in Progress

**Guita╭** is still a work in progress. Some areas that need attention include:

- 🧪 **Testing** – many parts of the app are experimental and require thorough testing, especially on large repositories.  
- 🔑 **Authentication experience** – handling SSH/HTTPS credentials could be customizable.  

Use it with caution, and feel free to report issues or contribute improvements!

---

## 🎨 Features

- 🖼️ **Beautiful graph rendering** – visualize commit history clearly.  
- 🚀 **Blazing-fast traversal** – works smoothly with very large repositories.  
- ⏩ **Immediate jumps** – move through history without waiting.  
- 🎨 **Vibrant, terminal-friendly colors** – easy on the eyes for long coding sessions.  
- 🧰 **Built-in diff viewer** – inspect changes without leaving the terminal.  
- 🦀 **Written in Rust** – safety, speed, and fun.  
- 🖥️ **Pure TUI experience** – ratatui based rendering.

---

## 🚀 Installation

Clone the repo and build with Cargo:

```bash
git clone https://github.com/asinglebit/guitar.git
cd guitar
cargo build --release path-to-your-repo
```
---

## 🖼️ Screenshots

<img width="1920" height="1008" alt="untitled" src="https://github.com/user-attachments/assets/5e175648-efc5-46a4-8fc1-6dda4c709d8e" />
<img width="1920" height="1080" alt="1" src="https://github.com/user-attachments/assets/87db026a-f419-46e3-8f20-f6389f3fa967" />
<img width="1920" height="1080" alt="2" src="https://github.com/user-attachments/assets/6cfbc5c0-222c-437d-a569-870446ed35ed" />
<img width="1920" height="1080" alt="3" src="https://github.com/user-attachments/assets/933a695d-5cec-4c82-8ef0-902cbcc1125b" />
<img width="1920" height="1080" alt="4" src="https://github.com/user-attachments/assets/67d6c13c-ff7e-4e97-8bb4-36228461c151" />
<img width="1920" height="1080" alt="5" src="https://github.com/user-attachments/assets/edc667bd-fb27-4b4c-8a4b-03ace73904a9" />
<img width="1920" height="1080" alt="6" src="https://github.com/user-attachments/assets/93c4e948-e3f3-49dd-aa63-a6fef5f6c1c4" />
<img width="1920" height="1080" alt="7" src="https://github.com/user-attachments/assets/b1534ac9-15a0-406d-97d4-8e15205b2d8d" />

---

## 📝 TODOs

Some things I still need to implement:

- 🖥️ **Logging window** – show detailed logs and messages in-app.  
- 🔐 **Credential manager** – smooth handling of SSH/HTTPS credentials.  
- 🌐 **Manage remotes** – add, remove, and inspect Git remotes.  
- 🧪 **Add tests** – improve reliability and catch edge cases.  
- ⚙️ **Git features**:
    - Create branch  
    - Delete branch  
    - Rename branch  
    - Pull branch  
    - Rebase branch  
    - Merge branch  
    - Cherry-pick commit  
    - Stash  
    - Pop  
    - Jump to parent commit
