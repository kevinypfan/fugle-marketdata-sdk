package tw.com.fugle.marketdata;

import org.junit.jupiter.api.*;
import static org.junit.jupiter.api.Assertions.*;
import static org.junit.jupiter.api.Assumptions.*;

import java.lang.reflect.Method;
import java.lang.reflect.Field;

/**
 * Response Compatibility Tests
 *
 * Validates response object structure matches expected Fugle API format.
 * Uses reflection for structural tests (no native library required).
 */
public class ResponseCompatibilityTest {

    private static boolean nativeLibraryAvailable = false;

    @BeforeAll
    static void checkNativeLibrary() {
        try {
            try (FugleRestClient client = FugleRestClient.builder()
                    .apiKey("test-key")
                    .build()) {
                nativeLibraryAvailable = true;
            }
        } catch (UnsatisfiedLinkError | NoClassDefFoundError e) {
            nativeLibraryAvailable = false;
        } catch (Exception e) {
            nativeLibraryAvailable = true;
        }
    }

    private void assumeNativeLibraryAvailable() {
        assumeTrue(nativeLibraryAvailable, "Native library not available. Build with: cargo build -p marketdata-uniffi --release");
    }

    // ========== Quote Response Structure ==========

    @Test
    @DisplayName("Quote record has symbol field")
    void quoteHasSymbolField() {
        // Check if Quote record/class exists with symbol method
        try {
            Class<?> quoteClass = Class.forName("uniffi.marketdata_uniffi.Quote");
            Method symbolMethod = quoteClass.getMethod("symbol");
            assertNotNull(symbolMethod, "Quote should have symbol() method");
            assertEquals(String.class, symbolMethod.getReturnType());
        } catch (ClassNotFoundException e) {
            fail("Quote class not found in uniffi bindings");
        } catch (NoSuchMethodException e) {
            fail("Quote.symbol() method not found");
        }
    }

    @Test
    @DisplayName("Quote record has date field")
    void quoteHasDateField() {
        try {
            Class<?> quoteClass = Class.forName("uniffi.marketdata_uniffi.Quote");
            Method dateMethod = quoteClass.getMethod("date");
            assertNotNull(dateMethod, "Quote should have date() method");
            assertEquals(String.class, dateMethod.getReturnType());
        } catch (ClassNotFoundException e) {
            fail("Quote class not found");
        } catch (NoSuchMethodException e) {
            fail("Quote.date() method not found");
        }
    }

    @Test
    @DisplayName("Quote record has expected fields")
    void quoteHasExpectedFields() {
        try {
            Class<?> quoteClass = Class.forName("uniffi.marketdata_uniffi.Quote");

            // Required fields
            assertNotNull(quoteClass.getMethod("symbol"), "Quote should have symbol()");
            assertNotNull(quoteClass.getMethod("date"), "Quote should have date()");

            // Optional fields (should exist in type)
            assertNotNull(quoteClass.getMethod("name"), "Quote should have name()");
            assertNotNull(quoteClass.getMethod("exchange"), "Quote should have exchange()");
            assertNotNull(quoteClass.getMethod("market"), "Quote should have market()");
        } catch (ClassNotFoundException e) {
            fail("Quote class not found");
        } catch (NoSuchMethodException e) {
            fail("Quote missing expected method: " + e.getMessage());
        }
    }

    // ========== Ticker Response Structure ==========

    @Test
    @DisplayName("Ticker record has symbol field")
    void tickerHasSymbolField() {
        try {
            Class<?> tickerClass = Class.forName("uniffi.marketdata_uniffi.Ticker");
            Method symbolMethod = tickerClass.getMethod("symbol");
            assertNotNull(symbolMethod, "Ticker should have symbol() method");
            assertEquals(String.class, symbolMethod.getReturnType());
        } catch (ClassNotFoundException e) {
            fail("Ticker class not found");
        } catch (NoSuchMethodException e) {
            fail("Ticker.symbol() method not found");
        }
    }

    @Test
    @DisplayName("Ticker record has expected fields")
    void tickerHasExpectedFields() {
        try {
            Class<?> tickerClass = Class.forName("uniffi.marketdata_uniffi.Ticker");

            assertNotNull(tickerClass.getMethod("symbol"), "Ticker should have symbol()");
            assertNotNull(tickerClass.getMethod("date"), "Ticker should have date()");
            assertNotNull(tickerClass.getMethod("name"), "Ticker should have name()");
        } catch (ClassNotFoundException e) {
            fail("Ticker class not found");
        } catch (NoSuchMethodException e) {
            fail("Ticker missing expected method: " + e.getMessage());
        }
    }

    // ========== Trades Response Structure ==========

    @Test
    @DisplayName("TradesResponse record has expected fields")
    void tradesResponseHasExpectedFields() {
        try {
            Class<?> tradesClass = Class.forName("uniffi.marketdata_uniffi.TradesResponse");

            assertNotNull(tradesClass.getMethod("symbol"), "TradesResponse should have symbol()");
            assertNotNull(tradesClass.getMethod("date"), "TradesResponse should have date()");
            assertNotNull(tradesClass.getMethod("data"), "TradesResponse should have data()");
        } catch (ClassNotFoundException e) {
            fail("TradesResponse class not found");
        } catch (NoSuchMethodException e) {
            fail("TradesResponse missing expected method: " + e.getMessage());
        }
    }

    // ========== Integration Response Tests ==========

    @Test
    @Tag("integration")
    @DisplayName("Quote response has required fields (live)")
    void quoteResponseHasRequiredFields() {
        assumeNativeLibraryAvailable();

        String apiKey = System.getenv("FUGLE_API_KEY");
        assumeTrue(apiKey != null && !apiKey.isEmpty(), "FUGLE_API_KEY not set");

        try (FugleRestClient client = FugleRestClient.builder()
                .apiKey(apiKey)
                .build()) {

            var quote = client.stock().intraday().getQuote("2330");

            assertNotNull(quote);
            assertNotNull(quote.symbol(), "Quote must have symbol");
            assertEquals("2330", quote.symbol());
            assertNotNull(quote.date(), "Quote must have date");
            assertFalse(quote.date().isEmpty(), "Quote date should not be empty");
        }
    }

    @Test
    @Tag("integration")
    @DisplayName("Ticker response has required fields (live)")
    void tickerResponseHasRequiredFields() {
        assumeNativeLibraryAvailable();

        String apiKey = System.getenv("FUGLE_API_KEY");
        assumeTrue(apiKey != null && !apiKey.isEmpty(), "FUGLE_API_KEY not set");

        try (FugleRestClient client = FugleRestClient.builder()
                .apiKey(apiKey)
                .build()) {

            var ticker = client.stock().intraday().getTicker("2330");

            assertNotNull(ticker);
            assertNotNull(ticker.symbol(), "Ticker must have symbol");
            assertEquals("2330", ticker.symbol());
            assertNotNull(ticker.name(), "Ticker must have name");
            assertFalse(ticker.name().isEmpty(), "Ticker name should not be empty");
        }
    }

    @Test
    @Tag("integration")
    @DisplayName("Trades response has required fields (live)")
    void tradesResponseHasRequiredFields() {
        assumeNativeLibraryAvailable();

        String apiKey = System.getenv("FUGLE_API_KEY");
        assumeTrue(apiKey != null && !apiKey.isEmpty(), "FUGLE_API_KEY not set");

        try (FugleRestClient client = FugleRestClient.builder()
                .apiKey(apiKey)
                .build()) {

            var trades = client.stock().intraday().getTrades("2330");

            assertNotNull(trades);
            assertNotNull(trades.symbol(), "Trades must have symbol");
            assertEquals("2330", trades.symbol());
            assertNotNull(trades.date(), "Trades must have date");
            assertNotNull(trades.data(), "Trades must have data array");
        }
    }
}
