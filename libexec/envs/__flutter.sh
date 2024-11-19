#!/usr/bin/env bash
#
# __flutter.sh
# realm
# 
# Author: Wess Cope (me@wess.io)
# Created: 12/21/2021
# 
# Copywrite (c) 2021 Wess.io
#
OUT_DIR="$(pwd)/.vscode/launch.json"
PUBSPEC="$(pwd)/pubspec.yaml"

define LAUNCH_CONFIG <<EOF
  {
    "version": "0.2.0",
    "configurations": [
      {
        "name": "$PROJECT",
        "request": "launch",
        "type": "dart",
        "args": [
          "--no-sound-null-safety",
          "--pub"
        ]
      }
    ]
  }
EOF

define FLUTTER_IGNORE <<EOF

### Flutter ###
# Flutter/Dart/Pub related
**/doc/api/
.dart_tool/
.flutter-plugins
.flutter-plugins-dependencies
.fvm/
.packages
.pub-cache/
.pub/
build/
coverage/
lib/generated_plugin_registrant.dart

# Android related
**/android/**/gradle-wrapper.jar
**/android/.gradle
**/android/captures/
**/android/gradlew
**/android/gradlew.bat
**/android/key.properties
**/android/local.properties
**/android/**/GeneratedPluginRegistrant.java

# iOS/XCode related
**/ios/**/*.mode1v3
**/ios/**/*.mode2v3
**/ios/**/*.moved-aside
**/ios/**/*.pbxuser
**/ios/**/*.perspectivev3
**/ios/**/*sync/
**/ios/**/.sconsign.dblite
**/ios/**/.tags*
**/ios/**/.vagrant/
**/ios/**/DerivedData/
**/ios/**/Icon?
**/ios/**/Pods/
**/ios/**/.symlinks/
**/ios/**/profile
**/ios/**/xcuserdata
**/ios/.generated/
**/ios/Flutter/.last_build_id
**/ios/Flutter/App.framework
**/ios/Flutter/Flutter.framework
**/ios/Flutter/Flutter.podspec
**/ios/Flutter/Generated.xcconfig
**/ios/Flutter/app.flx
**/ios/Flutter/app.zip
**/ios/Flutter/flutter_assets/
**/ios/Flutter/flutter_export_environment.sh
**/ios/ServiceDefinitions.json
**/ios/Runner/GeneratedPluginRegistrant.*

# Exceptions to above rules.
!**/ios/**/default.mode1v3
!**/ios/**/default.mode2v3
!**/ios/**/default.pbxuser
!**/ios/**/default.perspectivev3
!/packages/flutter_tools/test/data/dart_dependencies_test/**/.packages


EOF

if [[ -f "$PUBSPEC" ]]; then
  echo "Setting up for Flutter/Dart development..." | status

  if [ ! -f $OUT_DIR ]; then
    echo "Creating initial VSCode launch settings for flutter..." | status
    
    _dir="$(dirname -- "$OUT_DIR")"
    [ -d "$_dir" ] || mkdir -p -- "$_dir"
    touch -- "$OUT_DIR"


    echo "$LAUNCH_CONFIG" >> $OUT_DIR
  fi

  GIT_IGNORE="$(pwd)/.gitignore"

  if [ ! -f $GIT_IGNORE ]; then
    source ${SCRIPT_ENVS_DIR}/__gitignore.sh
  fi

  echo "$FLUTTER_IGNORE" >> .gitignore
fi

unset OUT_DIR
unset PUBSPEC
unset LAUNCH_CONFIG
