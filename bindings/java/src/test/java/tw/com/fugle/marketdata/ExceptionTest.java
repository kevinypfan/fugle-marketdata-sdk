package tw.com.fugle.marketdata;

import org.junit.jupiter.api.*;
import static org.junit.jupiter.api.Assertions.*;

import java.lang.reflect.Method;

/**
 * Tests for Fugle exception hierarchy.
 *
 * <p>All tests are structural (using reflection) and pass without native library.
 */
public class ExceptionTest {

    // ========== Type Hierarchy Tests ==========

    @Test
    @DisplayName("FugleException is a RuntimeException")
    void fugleExceptionIsRuntimeException() {
        assertNotNull(FugleException.class);
        assertTrue(RuntimeException.class.isAssignableFrom(FugleException.class));
    }

    @Test
    @DisplayName("ApiException extends FugleException")
    void apiExceptionExtendsFugleException() {
        assertNotNull(ApiException.class);
        assertTrue(FugleException.class.isAssignableFrom(ApiException.class));
    }

    @Test
    @DisplayName("RateLimitException extends ApiException")
    void rateLimitExceptionExtendsApiException() {
        assertNotNull(RateLimitException.class);
        assertTrue(ApiException.class.isAssignableFrom(RateLimitException.class));
    }

    @Test
    @DisplayName("AuthException extends FugleException")
    void authExceptionExtendsFugleException() {
        assertNotNull(AuthException.class);
        assertTrue(FugleException.class.isAssignableFrom(AuthException.class));
    }

    // ========== Constructor Tests ==========

    @Test
    @DisplayName("FugleException has message constructor")
    void fugleExceptionHasMessageConstructor() throws NoSuchMethodException {
        assertNotNull(FugleException.class.getConstructor(String.class));
    }

    @Test
    @DisplayName("FugleException has message and cause constructor")
    void fugleExceptionHasMessageCauseConstructor() throws NoSuchMethodException {
        assertNotNull(FugleException.class.getConstructor(String.class, Throwable.class));
    }

    @Test
    @DisplayName("ApiException has message and statusCode constructor")
    void apiExceptionHasConstructor() throws NoSuchMethodException {
        assertNotNull(ApiException.class.getConstructor(String.class, int.class));
    }

    @Test
    @DisplayName("RateLimitException has constructor with retryAfter")
    void rateLimitExceptionHasConstructor() throws NoSuchMethodException {
        assertNotNull(RateLimitException.class.getConstructor(String.class, int.class, Long.class));
    }

    @Test
    @DisplayName("AuthException has message constructor")
    void authExceptionHasMessageConstructor() throws NoSuchMethodException {
        assertNotNull(AuthException.class.getConstructor(String.class));
    }

    // ========== Method Tests ==========

    @Test
    @DisplayName("FugleException has from() static method")
    void fugleExceptionHasFromMethod() throws NoSuchMethodException {
        Method method = FugleException.class.getMethod("from", Throwable.class);
        assertNotNull(method);
        assertEquals(RuntimeException.class, method.getReturnType());
    }

    @Test
    @DisplayName("FugleException has unwrap() static method")
    void fugleExceptionHasUnwrapMethod() throws NoSuchMethodException {
        Method method = FugleException.class.getMethod("unwrap", Throwable.class);
        assertNotNull(method);
        assertEquals(Void.class, method.getReturnType());
    }

    @Test
    @DisplayName("ApiException has getStatusCode() method")
    void apiExceptionHasGetStatusCode() throws NoSuchMethodException {
        Method method = ApiException.class.getMethod("getStatusCode");
        assertNotNull(method);
        assertEquals(int.class, method.getReturnType());
    }

    @Test
    @DisplayName("RateLimitException has getRetryAfterSeconds() method")
    void rateLimitExceptionHasGetRetryAfterSeconds() throws NoSuchMethodException {
        Method method = RateLimitException.class.getMethod("getRetryAfterSeconds");
        assertNotNull(method);
        assertEquals(Long.class, method.getReturnType());
    }

    // ========== Behavior Tests ==========

    @Test
    @DisplayName("ApiException stores status code")
    void apiExceptionStoresStatusCode() {
        ApiException ex = new ApiException("Not found", 404);
        assertEquals(404, ex.getStatusCode());
        assertEquals("Not found", ex.getMessage());
    }

    @Test
    @DisplayName("RateLimitException stores retry after")
    void rateLimitExceptionStoresRetryAfter() {
        RateLimitException ex = new RateLimitException("Rate limit", 429, 60L);
        assertEquals(429, ex.getStatusCode());
        assertEquals(60L, ex.getRetryAfterSeconds());
        assertEquals("Rate limit", ex.getMessage());
    }

    @Test
    @DisplayName("RateLimitException handles null retry after")
    void rateLimitExceptionHandlesNullRetryAfter() {
        RateLimitException ex = new RateLimitException("Rate limit", 429, null);
        assertEquals(429, ex.getStatusCode());
        assertNull(ex.getRetryAfterSeconds());
    }

    @Test
    @DisplayName("AuthException preserves message")
    void authExceptionPreservesMessage() {
        AuthException ex = new AuthException("Invalid API key");
        assertEquals("Invalid API key", ex.getMessage());
    }

    @Test
    @DisplayName("FugleException.from() wraps generic exceptions")
    void fromWrapsGenericExceptions() {
        Exception cause = new Exception("Original error");
        RuntimeException wrapped = FugleException.from(cause);

        assertInstanceOf(FugleException.class, wrapped);
        assertEquals("Original error", wrapped.getMessage());
        assertEquals(cause, wrapped.getCause());
    }

    @Test
    @DisplayName("FugleException.from() preserves FugleException")
    void fromPreservesFugleException() {
        FugleException original = new FugleException("Already wrapped");
        RuntimeException result = FugleException.from(original);

        assertSame(original, result);
    }

    @Test
    @DisplayName("FugleException.from() preserves RuntimeException subclasses")
    void fromPreservesRuntimeException() {
        IllegalArgumentException original = new IllegalArgumentException("Invalid arg");
        RuntimeException result = FugleException.from(original);

        assertSame(original, result);
    }

    @Test
    @DisplayName("FugleException hierarchy is serializable")
    void exceptionsAreSerializable() {
        assertTrue(java.io.Serializable.class.isAssignableFrom(FugleException.class));
        assertTrue(java.io.Serializable.class.isAssignableFrom(ApiException.class));
        assertTrue(java.io.Serializable.class.isAssignableFrom(RateLimitException.class));
        assertTrue(java.io.Serializable.class.isAssignableFrom(AuthException.class));
    }
}
