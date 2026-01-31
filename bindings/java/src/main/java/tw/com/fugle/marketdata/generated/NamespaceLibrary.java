package tw.com.fugle.marketdata.generated;


import com.sun.jna.Library;
import com.sun.jna.Native;

final class NamespaceLibrary {
  static synchronized String findLibraryName(String componentName) {
    String libOverride = System.getProperty("uniffi.component." + componentName + ".libraryOverride");
    if (libOverride != null) {
        return libOverride;
    }
    return "marketdata_uniffi";
  }

  static <Lib extends Library> Lib loadIndirect(String componentName, Class<Lib> clazz) {
    return Native.load(findLibraryName(componentName), clazz);
  }

  static void uniffiCheckContractApiVersion(UniffiLib lib) {
    // Get the bindings contract version from our ComponentInterface
    int bindingsContractVersion = 26;
    // Get the scaffolding contract version by calling the into the dylib
    int scaffoldingContractVersion = lib.ffi_marketdata_uniffi_uniffi_contract_version();
    if (bindingsContractVersion != scaffoldingContractVersion) {
        throw new RuntimeException("UniFFI contract version mismatch: try cleaning and rebuilding your project");
    }
  }

  static void uniffiCheckApiChecksums(UniffiLib lib) {
    if (lib.uniffi_marketdata_uniffi_checksum_func_new_rest_client_with_api_key() != ((short) 2560)) {
        throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
    }
    if (lib.uniffi_marketdata_uniffi_checksum_func_new_rest_client_with_bearer_token() != ((short) 30582)) {
        throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
    }
    if (lib.uniffi_marketdata_uniffi_checksum_func_new_rest_client_with_sdk_token() != ((short) 14209)) {
        throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
    }
    if (lib.uniffi_marketdata_uniffi_checksum_func_new_websocket_client() != ((short) 17568)) {
        throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
    }
    if (lib.uniffi_marketdata_uniffi_checksum_func_new_websocket_client_with_endpoint() != ((short) 15148)) {
        throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
    }
    if (lib.uniffi_marketdata_uniffi_checksum_method_futoptclient_intraday() != ((short) 43120)) {
        throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
    }
    if (lib.uniffi_marketdata_uniffi_checksum_method_futoptintradayclient_get_products() != ((short) 61510)) {
        throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
    }
    if (lib.uniffi_marketdata_uniffi_checksum_method_futoptintradayclient_get_quote() != ((short) 21333)) {
        throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
    }
    if (lib.uniffi_marketdata_uniffi_checksum_method_futoptintradayclient_get_ticker() != ((short) 30953)) {
        throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
    }
    if (lib.uniffi_marketdata_uniffi_checksum_method_futoptintradayclient_products_sync() != ((short) 8976)) {
        throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
    }
    if (lib.uniffi_marketdata_uniffi_checksum_method_futoptintradayclient_quote_sync() != ((short) 33593)) {
        throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
    }
    if (lib.uniffi_marketdata_uniffi_checksum_method_futoptintradayclient_ticker_sync() != ((short) 53319)) {
        throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
    }
    if (lib.uniffi_marketdata_uniffi_checksum_method_restclient_futopt() != ((short) 65348)) {
        throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
    }
    if (lib.uniffi_marketdata_uniffi_checksum_method_restclient_stock() != ((short) 18733)) {
        throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
    }
    if (lib.uniffi_marketdata_uniffi_checksum_method_stockclient_intraday() != ((short) 53228)) {
        throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
    }
    if (lib.uniffi_marketdata_uniffi_checksum_method_stockintradayclient_candles_sync() != ((short) 10535)) {
        throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
    }
    if (lib.uniffi_marketdata_uniffi_checksum_method_stockintradayclient_get_candles() != ((short) 20034)) {
        throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
    }
    if (lib.uniffi_marketdata_uniffi_checksum_method_stockintradayclient_get_quote() != ((short) 64785)) {
        throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
    }
    if (lib.uniffi_marketdata_uniffi_checksum_method_stockintradayclient_get_ticker() != ((short) 26620)) {
        throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
    }
    if (lib.uniffi_marketdata_uniffi_checksum_method_stockintradayclient_get_trades() != ((short) 48306)) {
        throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
    }
    if (lib.uniffi_marketdata_uniffi_checksum_method_stockintradayclient_get_volumes() != ((short) 41478)) {
        throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
    }
    if (lib.uniffi_marketdata_uniffi_checksum_method_stockintradayclient_quote_sync() != ((short) 24390)) {
        throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
    }
    if (lib.uniffi_marketdata_uniffi_checksum_method_stockintradayclient_ticker_sync() != ((short) 22635)) {
        throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
    }
    if (lib.uniffi_marketdata_uniffi_checksum_method_stockintradayclient_trades_sync() != ((short) 4040)) {
        throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
    }
    if (lib.uniffi_marketdata_uniffi_checksum_method_stockintradayclient_volumes_sync() != ((short) 8850)) {
        throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
    }
    if (lib.uniffi_marketdata_uniffi_checksum_method_websocketclient_connect() != ((short) 52173)) {
        throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
    }
    if (lib.uniffi_marketdata_uniffi_checksum_method_websocketclient_disconnect() != ((short) 33142)) {
        throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
    }
    if (lib.uniffi_marketdata_uniffi_checksum_method_websocketclient_is_connected() != ((short) 53625)) {
        throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
    }
    if (lib.uniffi_marketdata_uniffi_checksum_method_websocketclient_subscribe() != ((short) 63126)) {
        throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
    }
    if (lib.uniffi_marketdata_uniffi_checksum_method_websocketclient_unsubscribe() != ((short) 9652)) {
        throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
    }
    if (lib.uniffi_marketdata_uniffi_checksum_method_websocketlistener_on_connected() != ((short) 56842)) {
        throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
    }
    if (lib.uniffi_marketdata_uniffi_checksum_method_websocketlistener_on_disconnected() != ((short) 54477)) {
        throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
    }
    if (lib.uniffi_marketdata_uniffi_checksum_method_websocketlistener_on_message() != ((short) 54327)) {
        throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
    }
    if (lib.uniffi_marketdata_uniffi_checksum_method_websocketlistener_on_error() != ((short) 64085)) {
        throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
    }
    if (lib.uniffi_marketdata_uniffi_checksum_constructor_websocketclient_new() != ((short) 36225)) {
        throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
    }
    if (lib.uniffi_marketdata_uniffi_checksum_constructor_websocketclient_new_with_endpoint() != ((short) 35702)) {
        throw new RuntimeException("UniFFI API checksum mismatch: try cleaning and rebuilding your project");
    }
  }
}

// Define FFI callback types
