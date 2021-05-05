/***
 * @brief This C header file exposes the FFI defined inside the ffi crate.
 */

#include <stdint.h>

typedef struct wallet wallet_t;

/**
 * @brief Struct that represents a signature.
 */
typedef struct {
  /**
   * @brief The length of the signature.
   */
  uint32_t len;
  /**
   * @brief The data signature.
   */
  uint8_t* data;
} signature_t;

/**
 * @brief Creates a random 24 words mnemonic.
 * @return Returns the generated mnemonic or NULL in case of error.
 * The caller must take care of releasing the returned mnemonic with the
 * cstring_free function.
 * In case of error the error cause can be obtained using the error_message_utf8
 * function.
 */
char* wallet_random_mnemonic();

/**
 * @brief Free a string.
 * @param str: Pointer to the string to free.
 */
void cstring_free(char* str);

/**
 * @brief Derive a Secp256k1 key pair from the given mnemonic and derivation_path.
 * @param mnemonic: The wallet mnemonic.
 * @param derivation_path: The derivation path used to derive the keys from the mnemonic.
 * @return Returns a pointer to a valid wallet or NULL on error.
 * The caller must take care of freeing the returned wallet instance with
 * the wallet_free function.
 * In case of error the error cause can be obtained using the error_message_utf8
 * function.
 */
wallet_t* wallet_from_mnemonic(const char* mnemonic, const char* derivation_path);

/**
 * @brief Free a wallet instance.
 * @param wallet: The wallet to free.
 */
void wallet_free(wallet_t* wallet);

/**
 * @brief Gets the bec32 address associated to the wallet.
 * @param wallet: Pointer to the wallet instance.
 * @param hrp: The address human readable part.
 * @return Returns the bech32 address associated to the wallet on success or
 * NULL on error.
 * The caller must take care of freeing the returned address with the
 * cstring_free function.
 * In case of error the error cause can be obtained using the error_message_utf8
 * function.
 */
char* wallet_get_bech32_address(wallet_t *wallet, const char* hrp);

/**
 * @brief Gets secp256 public key from the wallet.
 * @param wallet: Pointer to the wallet instance.
 * @param compressed: True to get the public key in a compressed format, false otherwise.
 * @param out_buffer: Pointer where will be stored the public key
 * @param size: Size of out_buffer.
 * @return Returns the number of bytes wrote inside out_buffer on success,
 * -1 if the provided arguments are invalid or -2 if the public key don't fit
 * into out_buffer.
 */
int wallet_get_public_key(wallet_t *wallet, uint32_t compressed, uint8_t *out_buffer, int size);

/**
 * @brief Performs the signature of the provided data.
 * @param wallet: Pointer to the wallet instance.
 * @param data: The data to sign.
 * @param len: The length of the data to sign.
 * @return Returns a pointer to a signature_t instance on success, NULL on error.
 * The caller must take care of freeing the returned signature with the
 * wallet_sign_free function.
 * In case of error the error cause can be obtained using the error_message_utf8
 * function.
 */
signature_t* wallet_sign(wallet_t *wallet, const uint8_t* data, uint32_t len);

/**
 * @brief Free a signature instance.
 * @param signature: Pointer to the signature to free.
 */
void wallet_sign_free(signature_t* signature);

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
