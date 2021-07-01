#ifndef CRW_PREFERENCES_H
#define CRW_PREFERENCES_H

#include <stdarg.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>

/**
 * @brief Sets the application directory where will be stored the configurations.
 * @param name On windows, macOS and linux should be only the name of the directory that will
 * be created inside the current user configurations directory.
 * On Android instead since is not possible to obtain the `appData` directory at runtime
 * must be an absolute path to a directory where the application can read and write.
 * @return Returns 0 on success -1 on error.
 * In case of error the error cause can be obtained using the error_message_utf8
 * function.
 */
int set_preferences_app_dir(const char *name);

/**
 * @brief Checks if exist a preferences with the provided name.
 * @param name The preference name.
 * @return Return true if the preference exist, false otherwise.
 */
bool preferences_exist(const char *name);

/**
 * @brief Deletes a preferences with the provided name.
 * @param name Name of the preference to delete.
 */
void preferences_delete(const char *name);

/**
 * @brief Creates a new preferences with the provided name or
 * loads a previously created preferences with the same name.
 * @param name The preferences name, can contains only ascii alphanumeric chars or -, _.
 * @return Returns a valid pointer on success or nullptr if an error occurred.
 * In case of error the error cause can be obtained using the error_message_utf8
 * function.
 */
void *preferences(const char *name);

/**
 * @brief Creates a new encrypted preferences with the provided name or
 * loads a previously created preferences with the same name.
 * @param name The preferences name, can contains only ascii alphanumeric chars or -, _.
 * @param password The password used to secure the preferences.
 * @return Returns a valid pointer on success or nullptr if an error occurred.
 * In case of error the error cause can be obtained using the error_message_utf8
 * function.
 */
void *encrypted_preferences(const char *name,
                            const char *password);

/**
 * @biref Release all the resources owned by a preferences instance.
 * @param preferences Pointer to the preference instance to free.
 */
void preferences_free(void *preferences);

/**
 * @brief Gets an i32 from the preferences.
 * @param preferences pointer to the preferences from which will be extracted the value.
 * @param key name of the preference that will be loaded.
 * @param out pointer where will be stored the value.
 * @return Returns 0 on success -1 if the requested value is not
 * present into the preferences or -2 if one or more of the provided arguments is invalid.
 * In case of error the error cause can be obtained using the error_message_utf8
 * function.
 */
int preferences_get_i32(const void *preferences, const char *key, int32_t *out);

/**
 * @brief  Puts an i32 into the preferences.
 * @param preferences pointer to the preferences where will be stored the value.
 * @param key name of the preference that will be stored.
 * @param value the value that will be stored.
 * @return Returns 0 on success or -1 on error.
 * In case of error the error cause can be obtained using the error_message_utf8
 * function.
 */
int preferences_put_i32(void *preferences, const char *key, int32_t value);

/**
 * @brief Gets a string from the preferences.
 * @param preferences pointer to the preferences from which will be extracted the value.
 * @param key name of the preference that will be loaded.
 * @param out_buf pointer where will be stored the value.
 * @param buf_len maximum number of bytes that can be used from `out_buf`
 * @return Returns the number of bytes that would have been written if `out_buf`
 * had been sufficiently large,0 if the value is not present into the preferences or -1 on error.
 * In case of error the error cause can be obtained using the error_message_utf8
 * function.
 */
int preferences_get_string(const void *preferences,
                           const char *key,
                           unsigned char *out_buf,
                           size_t len);

/**
 * @brief Puts a string into the preferences.
 * @param preferences pointer to the preferences from which will be extracted the value.
 * @param key name of the preference that will be loaded.
 * @param value the value that will be stored.
 * @return Returns 0 on success -1 on error.
 * In case of error the error cause can be obtained using the error_message_utf8
 * function.
 */
int preferences_put_string(void *preferences, const char *key, const char *value);

/**
 * @brief Gets a bool from the preferences.
 * @param preferences pointer to the preferences from which will be extracted the value.
 * @param key name of the preference that will be loaded.
 * @param out pointer where will be stored the value.
 * @return Returns 0 on success -1 if the requested value is not present into the
 * preferences or -2 if one or more of the provided arguments is invalid.
 * In case of error the error cause can be obtained using the error_message_utf8
 * function.
 */
int preferences_get_bool(const void *preferences, const char *key, bool *out);

/**
 * @brief Puts a bool into the preferences.
 * @param preferences pointer to the preferences where will be stored the value.
 * @param key name of the preference that will be stored.
 * @param value the value that will be stored.
 * @return Returns 0 on success or -1 on error.
 * In case of error the error cause can be obtained using the error_message_utf8
 * function.
 */
int preferences_put_bool(void *preferences, const char *key, bool value);

/**
 * @brief Gets an array of bytes from the preferences.
 * @param preferences pointer to the preferences from which will be extracted the value.
 * @param key name of the preference that will be loaded.
 * @param out_buf pointer where will be stored the value.
 * @param buf_len maximum number of bytes that can be used from `out_buf`
 * @return  Returns the number of bytes that would have been written if `out_buf`
 * had been sufficiently large, 0 if the value is not present into the preferences or -1 on error.
 * In case of error the error cause can be obtained using the error_message_utf8
 * function.
 */
int preferences_get_bytes(const void *preferences,
                          const char *key,
                          uint8_t *out_buf,
                          size_t buf_len);

/**
 * @brief Store an array of bytes into the preferences.
 * @param preferences pointer to the preferences from which will be extracted the value.
 * @param key name of the preference that will be stored.
 * @param value array that will be stored into the preferences.
 * @param len length of `value`.
 * @return Returns 0 on success, -1 on error.
 * In case of error the error cause can be obtained using the error_message_utf8
 * function.
 */
int preferences_put_bytes(void *preferences,
                          const char *key,
                          const uint8_t *value,
                          size_t len);

/**
 * @brief Delete all the preferences currently loaded from the provided
 * preferences instance.
 * @param preferences pointer to the preferences instance.
 * @return Returns 0 on success or -1 on error.
 * In case of error the error cause can be obtained using the error_message_utf8
 * function.
 */
int preferences_clear(void *preferences);

/**
 * @brief Delete all the preferences currently loaded and also the one stored
 * into the device storage from the provided preferences instance
 * @param preferences pointer to the preferences instance.
 * @return Returns 0 on success or -1 on error.
 * In case of error the error cause can be obtained using the error_message_utf8
 * function.
 */
int preferences_erase(void *preferences);

/**
 * @brief Saves the preferences into the device disk.
 * @param preferences pointer to the preferences instance.
 * @return Returns 0 on success or -1 on error.
 * In case of error the error cause can be obtained using the error_message_utf8
 * function.
 */
int preferences_save(void *preferences);

/**
 * @brief Clears the last error.
 */
void clear_last_error();

/**
 * @brief Gets the last error message length.
 */
int last_error_length();

/**
 * @brief Gets the last error message as UTF-8 encoded string.
 * @param out_buf: Pointer where will be stored the error message.
 * @param buf_size: Size of out_buf.
 * @return Returns the number of bytes wrote into out_buf or -1 on error.
 */
int error_message_utf8(char *out_buf, int buf_size);

#endif /* CRW_PREFERENCES_H */
