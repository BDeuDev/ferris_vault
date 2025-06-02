# 🦀 Ferris Vault

Ferris Vault is a desktop application written in Rust for generating, copying, saving, and viewing passwords securely and easily.

![screenshot](assets/screenshot.png)

## ✨ Features

- Custom password generation:
  - Adjustable length.
  - Option to include uppercase letters, numbers, and symbols.
- View or hide saved passwords.
- Copy passwords to clipboard with a single click.
- Delete saved passwords.
- Auto-save to local JSON file (`passwords.json`).

## 🛠️ Built With

- [Rust](https://www.rust-lang.org/)
- [egui](https://github.com/emilk/egui) + [eframe](https://docs.rs/eframe/latest/eframe/)
- [serde](https://serde.rs/) for serialization
- [rand](https://docs.rs/rand/) for randomness

## 🚀 Getting Started

### 1. Clone the repository

```bash
git clone https://github.com/BDeuDev/ferris_vault.git
cd ferris_vault
```

### 2. Run the app

```bash
cargo run
```

### 3. Build a release binary (optional)

```bash
cargo build --release
```

## 📁 Project Structure

```bash
ferris_vault/
├── src/
│   └── main.rs            # Entry point of the application; sets up the GUI and handles app state.
├── assets/
│   └── icon.png           # App icon used in the window or binary packaging.
│   └── screenshot.png     # Optional screenshot to show the UI (used in README).
├── crypto/                # Handles encryption, decryption, and key management logic.
│   └── decrypt.rs         # Contains logic to decrypt stored password data using AES-GCM.
│   └── encrypt.rs         # Contains logic to encrypt password data before saving.
│   └── master_key.rs      # Manages generation, storage, and validation of the master key.
│   └── mod.rs             # Module file to expose submodules (encrypt, decrypt, etc.) to the rest of the app.
│   └── utils.rs           # Shared crypto utilities, like key derivation or random generation helpers.
├── master_key.json        # Auto-generated file that stores the encrypted master key (local, secure).
├── passwords.json         # Saved encrypted passwords (auto-generated and updated at runtime).
├── Cargo.toml             # Rust manifest file declaring dependencies and project metadata.
└── README.md              # Project documentation with usage, features, and contribution info.
```

## 🔐 Security Notes

- All passwords are encrypted before being saved locally.
- A **master key** is required to view or decrypt saved passwords.
- The `master_key.json` file is auto-generated on first launch. Keep this file safe, as it's essential for decrypting your passwords.
- The app performs all operations **locally** – no cloud storage, no external communication.

## 🧵 Multithreading & Performance

To ensure a smooth and responsive user interface, Ferris Vault uses multi-threading for cryptographic operations:

- Password encryption and decryption are executed in separate background threads.
- These threads communicate with the UI thread using crossbeam-channel.
- This design avoids UI blocking, enabling a fluid and responsive experience even during heavy operations.

## 📦 Dependencies

Here are the main crates used in the project:

- [`eframe`](https://crates.io/crates/eframe): Framework for building native GUI apps with `egui`.
- [`aes-gcm`](https://crates.io/crates/aes-gcm): Authenticated encryption (AEAD) used for secure password encryption.
- [`aes`](https://crates.io/crates/aes): Low-level AES block cipher used internally.
- [`pbkdf2`](https://crates.io/crates/pbkdf2): Derives secure keys from passwords using the PBKDF2 algorithm.
- [`sha2`](https://crates.io/crates/sha2): SHA-2 hashing algorithm used in conjunction with PBKDF2.
- [`rand`](https://crates.io/crates/rand): Used to generate random secure passwords.
- [`base64`](https://crates.io/crates/base64): Encodes encrypted data into base64 for safe storage.
- [`serde`](https://crates.io/crates/serde) + [`serde_json`](https://crates.io/crates/serde_json): For serializing and deserializing password data and the master key.
- [`crossbeam-channel`](https://crates.io/crates/crossbeam-channel): Thread-safe communication between GUI and logic components.

## 🧪 Development Notes

- Supports hot-reloading via `cargo run`.
- Settings and saved data are preserved across sessions via local JSON files.
- Designed to be lightweight and portable.
- Heavy cryptographic operations (encryption/decryption) are offloaded to background threads to avoid blocking the main UI thread.

## 🖥️ Platform Support

- ✅ Windows  
- ✅ Linux  
- 🟡 macOS (untested but should work)

## 💡 Ideas for Future Improvements

- Add option to change master key.
- Add password categories or tags.
- Export/import passwords.
- Integrate with system tray.
- Add search/filter functionality.

## 🤝 Contributing

Pull requests are welcome! If you have suggestions for improvements or new features, feel free to open an issue or fork the project.

## 📄 License

This project is licensed under the **MIT License** – see the [LICENSE](./LICENSE) file for details.
