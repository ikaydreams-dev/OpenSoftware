# EnglishCode Mobile - iOS & Android Support

## Overview

EnglishCode now **compiles to native iOS and Android apps** using React Native. Write your app once in plain English, deploy to App Store and Google Play.

## Installation

```bash

# Install React Native dependencies
npm install -g react-native-cli

# For iOS (macOS only)
brew install cocoapods
xcode-select --install

# For Android
# Download Android Studio and set ANDROID_HOME
export ANDROID_HOME=$HOME/Library/Android/sdk
```


## Quick Start

### 1. Create Your App in English

```englishcode

create a page called "home" with title "My App"

add heading "Welcome!"
add paragraph "Built with EnglishCode"
add image from "logo.png" with alt "Logo"
add button labeled "Get Started"

render page "home"
```


### 2. Initialize Mobile Project

```bash

engcode mobile init MyApp --output ./mobile
```


### 3. Build from .eng File

```bash

engcode mobile build app.eng --platform both
```


This transpiles your English code to React Native (JavaScript/TypeScript).

### 4. Run on Simulators/Emulators

**iOS:**
```bash

engcode mobile run-ios
```


**Android:**
```bash

engcode mobile run-android
```


## Features

### Supported Components

| English | React Native Component |
|---------|----------------------|
| `add heading "Text"` | `<Text style={{fontSize: 32, fontWeight: 'bold'}}>` |
| `add paragraph "Text"` | `<Text style={{fontSize: 16}}>` |
| `add button labeled "X"` | `<TouchableOpacity><Text>` |
| `add input with type "email"` | `<TextInput />` |
| `add image from "url"` | `<Image source={{uri: ...}} />` |

### Styling

All components get automatic mobile-optimized styles:
- Responsive padding/margins
- Touch-friendly button sizes (44pt minimum)
- Platform-specific fonts
- Dark mode support (coming soon)

### Layout

EnglishCode uses `ScrollView` by default for mobile-friendly scrolling.

```englishcode

create a page called "profile"
add heading "Profile"
add image from "avatar.png" with alt "Avatar"
add paragraph "User bio here"
add button labeled "Edit Profile"
```


### Coming Soon

- [ ] Navigation between screens
- [ ] Async data fetching
- [ ] Native modules (Camera, GPS, etc.)
- [ ] Push notifications
- [ ] App icons & splash screens
- [ ] Release builds (IPA/APK/AAB)

## Building for Production

### iOS (.ipa for App Store)

```bash

cd mobile/ios
pod install
xcodebuild -workspace MyApp.xcworkspace \
 -scheme MyApp \
 -configuration Release \
 -archivePath build/MyApp.xcarchive \
 archive
```


Then export IPA using Xcode.

### Android (.apk/.aab for Google Play)

```bash

cd mobile/android
./gradlew assembleRelease # APK
./gradlew bundleRelease # AAB (recommended)
```


Output: `android/app/build/outputs/`

## Example: Full App

**input.eng:**
```englishcode

create a page called "login"

add heading "Sign In"
add input with type "email" and placeholder "Email"
add input with type "password" and placeholder "Password"
add button labeled "Login"

create a page called "home"
add heading "Dashboard"
add paragraph "Welcome back!"
add button labeled "Logout"
```


**Transpiles to React Native:**
```tsx

import React, { useState } from 'react';
import { View, Text, TextInput, TouchableOpacity, ScrollView, StyleSheet } from 'react-native';

export default function App() {
 return (
    <ScrollView style={styles.container}>

      <Text style={{fontSize: 32, fontWeight: 'bold'}}>Sign In</Text>

      <TextInput placeholder="Email" style={{borderWidth: 1, padding: 12}} />

      <TextInput placeholder="Password" secureTextEntry={true} style={{borderWidth: 1, padding: 12}} />

      <TouchableOpacity style={{backgroundColor: '#007bff', padding: 15}}>

        <Text style={{color: 'white', textAlign: 'center'}}>Login</Text>

      </TouchableOpacity>

    </ScrollView>

 );
}

const styles = StyleSheet.create({
 container: {
    flex: 1,

    backgroundColor: '#fff',

    padding: 20,

 },
});
```


## CLI Commands

| Command | Description |
|---------|-------------|
| `engcode mobile init <name>` | Create new React Native project |
| `engcode mobile build <file.eng>` | Transpile .eng to React Native |
| `engcode mobile run-ios` | Run on iOS simulator |
| `engcode mobile run-android` | Run on Android emulator |
| `engcode mobile list-devices` | Show available simulators |

## Architecture

```

┌─────────────────┐
│ .eng file │ (English source code)
└────────┬────────┘
         │

         ▼

┌─────────────────┐
│ Lexer → Parser │ (Parse to AST)
└────────┬────────┘
         │

         ▼

┌─────────────────┐
│ MobileTranspiler│ (Transpile AST → React Native)
└────────┬────────┘
         │

         ▼

┌─────────────────┐
│ App.tsx │ (React Native code)
└────────┬────────┘
         │

    ┌────┴─────┐

    ▼          ▼

┌────────┐ ┌────────┐
│ iOS │ │Android │
│ .ipa │ │ .apk │
└────────┘ └────────┘
```


## GitHub Workflow

Your `.eng` files stay in English on GitHub:

```bash

git add app.eng
git commit -m "Add mobile app"
git push
```


The transpiled React Native code is gitignored (in `mobile/` directory).

## Requirements

- **macOS 12+** for iOS builds
- **Node.js 18+** for React Native
- **Xcode 14+** for iOS
- **Android Studio** for Android
- **JDK 11+** for Android builds

## Testing on Physical Devices

**iOS:**
1. Connect iPhone via USB
2. Trust computer on device
3. `engcode mobile run-ios --device`

**Android:**
1. Enable Developer Mode + USB Debugging
2. Connect via USB
3. `adb devices` to verify
4. `engcode mobile run-android`

## Performance

EnglishCode mobile apps have the same performance as hand-written React Native:
- Native UI components (not web views)
- 60 FPS animations
- Small bundle size (~3-5MB)
- Fast startup time

## Support

Questions? Issues?
- GitHub: https://github.com/yourusername/englishcode
- Docs: https://englishcode.dev/mobile
- Discord: discord.gg/englishcode
