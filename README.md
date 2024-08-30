# <image src="./assets/ico-48.png" style="vertical-align: middle"> Emoji Picker 

This is a simple emoji picker that replaces the emoji picker in Windows ≥10, with **all** shortcuts. 

:warning: This is a work in progress and mainly a learning project.

## Demo

<center>

![Demo](./demo.gif)

</center>

## Running

* Tested on Windows 11 23H2, MSVC toolchain.

```bash
git clone --recurse-submodules git@github.com:Maeeen/emoji-picker.git
cd emoji-picker
cargo run
```

* For Windows 7/8 (i.e. demo), [Rust needs to be downgraded to 1.75](https://github.com/rustdesk/rustdesk/discussions/7503).
  * Moreover, [`rowan`](https://crates.io/crates/rowan) must be downgraded to 0.15.15: `cargo update rowan@0.15.16 --precise 0.15.15`

## Features

* Replaces the painfully slow and bloated Windows emoji picker (personal opinion.)
* You can tweak it to your liking (since this is fully public.)

### Goal

This executable should behave in the same manner as the Windows Emoji Picker, with the main goal of:
- having the same shortcut (achieved by using a `WH_KEYBOARD_LL` Windows hook)
- having the same behavior : focus to the main window is not lost when the picker is opened, and the picker closes when the focus is lost.

On top of that, the picker should be not CPU nor memory intensive. Currently, it uses around ~50MB of RAM and 0% CPU when idle. The `emoji-picker-hooker` is around ~10KB.

### Why?

When talking to some friends on Instagram web, I type relatively fast and… clicking on the emoji button… looking for the correct one… clicking on it… became annoying.

Every [emoji picker](https://google.com/search?q=emoji+picker+windows+github) copies the emoji to the clipboard (which is a rather okay solution), but I really liked the built-in picker on Windows.
So I made this! It does not _copy_ the emoji but directly types it. Focus on the main window is not lost. I wanted to learn Rust, and play a bit with the Win32 API.

## Crate features

All these features are for Windows only. They will have no effect if not running on Windows.

* `caret`: will place the window near the caret (cursor) position.
* `no-activate`/`key-redir`: will not activate the window when opened, focus will not be lost when the picker is opened.
* `key-shortcut`: will open the picker with the <kbd>Win</kbd> + <kbd>.</kbd> shortcut.
* `back-click`: will close the picker when the user clicks outside the picker.

Multi-platform features:

* `tray-icon`: will show a tray icon that can be used to open the picker.

## Targets

* The main target is Windows 11, but it should work on older versions of Windows as well. No need for administrator privileges.
* On Linux/macOS, there is no strict requirement for this but the missing implementations are:
  * [ ] The keyboard shortcut to open the picker. (<kbd>Win</kbd> + <kbd>.</kbd>) (`src/key_shortcut.rs`)
  * [ ] The “key interceptor” that prevents focus being made to the Emoji Picker window AND intercepts every key made while the picker is open.
  * [ ] The “key sender” that sends keys to the main window.

## TO-DO

* [ ] Better emoji look-ups
* [x] Emojis should be displayed in a grid
* [ ] Polish design
  * [ ] Add a *tooltip* to see the name of the emojis
* [ ] Better name everything (notably, features)
* [x] Emoji groups
* [ ] Should copy to clipboard if no text box is focused
* [ ] Skin tones
* [x] A tray-icon (maybe?) Not a good idea to have a process floating around without showing its existence to the user.
* [ ] Better readme and showcase
* [x] The search text input should be automatically focused on opening
* [x] `caret_locator` places the window in the screen's bbox
* [ ] Doing the TODOs in code
* [x] Lost focus closes the picker
* [ ] Customize emoji image source
  + Currently, the app uses [`jdecked/twemoji`](https://github.com/jdecked/twemoji) for the emojis along with the [`emojis` crate](https://crates.io/crates/emojis). In the future,
    it would be a good idea to not make it a submodule, as it requires manually updating the submodule (maybe?)
* [ ] Customize shortcuts (when opening the picker.)
* [ ] Maybe Linux support?