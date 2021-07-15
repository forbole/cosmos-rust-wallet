# Preferences package
This package take care of storing a set of preferences into the device storage.  
The preferences sets are stored in different locations based on the device os. The table below shows
where the preferences are stored based on the device's os.

|Platform | Location |  
| ------- | ---------------- |  
| Linux   | $XDG_CONFIG_HOME/{PREFERENCES_APP_DIR} or $HOME/.config/{PREFERENCES_APP_DIR} |  
| macOS   | $HOME/Library/Application Support/{PREFERENCES_APP_DIR} |  
| Windows |  C:\Users\\$USER\AppData\Roaming\{PREFERENCES_APP_DIR} |  
| Android | {PREFERENCES_APP_DIR} |  
| iOS     | {PREFERENCES_APP_DIR} |  
| Web | LocalStorage |  

`PREFERENCES_APP_DIR` is the value provided with the `set_preferences_app_dir`.  

**NOTE:** Since in iOS and Android is not possible to know the 
application data directory the full path must be provided from the user 
using the `set_preferences_app_dir` function.

