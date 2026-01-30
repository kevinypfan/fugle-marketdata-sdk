/**
 * API Compatibility Tests
 *
 * Verifies that the SDK structure matches the official @fugle/marketdata SDK.
 * These tests do NOT require network access or a valid API key - they only test
 * that the expected methods and properties exist with the correct types.
 */
const { RestClient, WebSocketClient } = require('../');

/**
 * Helper to check if a value is Promise-like (has .then and .catch methods)
 * This handles both native Promises and napi-rs Promises which may have different
 * constructors but behave identically.
 */
function isPromiseLike(value) {
  return value && typeof value.then === 'function' && typeof value.catch === 'function';
}

describe('API Compatibility', () => {
  describe('RestClient structure', () => {
    let client;

    beforeAll(() => {
      client = new RestClient('test-api-key');
    });

    test('RestClient constructor accepts API key', () => {
      expect(() => new RestClient('any-key')).not.toThrow();
    });

    test('RestClient has stock property', () => {
      expect(client.stock).toBeDefined();
    });

    test('RestClient has futopt property', () => {
      expect(client.futopt).toBeDefined();
    });

    describe('stock.intraday methods', () => {
      let intraday;

      beforeAll(() => {
        intraday = client.stock.intraday;
      });

      test('stock.intraday exists', () => {
        expect(intraday).toBeDefined();
      });

      test('quote method exists and is a function', () => {
        expect(typeof intraday.quote).toBe('function');
      });

      test('ticker method exists and is a function', () => {
        expect(typeof intraday.ticker).toBe('function');
      });

      test('candles method exists and is a function', () => {
        expect(typeof intraday.candles).toBe('function');
      });

      test('trades method exists and is a function', () => {
        expect(typeof intraday.trades).toBe('function');
      });

      test('volumes method exists and is a function', () => {
        expect(typeof intraday.volumes).toBe('function');
      });

      test('quote returns a Promise-like object', async () => {
        const result = intraday.quote('2330');
        expect(isPromiseLike(result)).toBe(true);
        // Wait for rejection and suppress it
        await expect(result).rejects.toThrow();
      });

      test('ticker returns a Promise-like object', async () => {
        const result = intraday.ticker('2330');
        expect(isPromiseLike(result)).toBe(true);
        await expect(result).rejects.toThrow();
      });

      test('candles returns a Promise-like object', async () => {
        const result = intraday.candles('2330', '5');
        expect(isPromiseLike(result)).toBe(true);
        await expect(result).rejects.toThrow();
      });

      test('trades returns a Promise-like object', async () => {
        const result = intraday.trades('2330');
        expect(isPromiseLike(result)).toBe(true);
        await expect(result).rejects.toThrow();
      });

      test('volumes returns a Promise-like object', async () => {
        const result = intraday.volumes('2330');
        expect(isPromiseLike(result)).toBe(true);
        await expect(result).rejects.toThrow();
      });
    });

    describe('futopt.intraday methods', () => {
      let intraday;

      beforeAll(() => {
        intraday = client.futopt.intraday;
      });

      test('futopt.intraday exists', () => {
        expect(intraday).toBeDefined();
      });

      test('quote method exists and is a function', () => {
        expect(typeof intraday.quote).toBe('function');
      });

      test('ticker method exists and is a function', () => {
        expect(typeof intraday.ticker).toBe('function');
      });

      test('candles method exists and is a function', () => {
        expect(typeof intraday.candles).toBe('function');
      });

      test('trades method exists and is a function', () => {
        expect(typeof intraday.trades).toBe('function');
      });

      test('volumes method exists and is a function', () => {
        expect(typeof intraday.volumes).toBe('function');
      });

      test('products method exists and is a function', () => {
        expect(typeof intraday.products).toBe('function');
      });

      test('products returns a Promise-like object', async () => {
        const result = intraday.products('FUTURE');
        expect(isPromiseLike(result)).toBe(true);
        await expect(result).rejects.toThrow();
      });
    });
  });

  describe('WebSocketClient structure', () => {
    let client;

    beforeAll(() => {
      client = new WebSocketClient('test-api-key');
    });

    test('WebSocketClient constructor accepts API key', () => {
      expect(() => new WebSocketClient('any-key')).not.toThrow();
    });

    test('WebSocketClient has stock property', () => {
      expect(client.stock).toBeDefined();
    });

    test('WebSocketClient has futopt property', () => {
      expect(client.futopt).toBeDefined();
    });

    describe('stock WebSocket methods', () => {
      let stock;

      beforeAll(() => {
        stock = client.stock;
      });

      test('on method exists and is a function', () => {
        expect(typeof stock.on).toBe('function');
      });

      test('connect method exists and is a function', () => {
        expect(typeof stock.connect).toBe('function');
      });

      test('subscribe method exists and is a function', () => {
        expect(typeof stock.subscribe).toBe('function');
      });

      test('unsubscribe method exists and is a function', () => {
        expect(typeof stock.unsubscribe).toBe('function');
      });

      test('disconnect method exists and is a function', () => {
        expect(typeof stock.disconnect).toBe('function');
      });

      test('isConnected is a boolean', () => {
        expect(typeof stock.isConnected).toBe('boolean');
      });

      test('isClosed is a boolean', () => {
        expect(typeof stock.isClosed).toBe('boolean');
      });

      test('initial state is disconnected', () => {
        expect(stock.isConnected).toBe(false);
      });
    });

    describe('futopt WebSocket methods', () => {
      let futopt;

      beforeAll(() => {
        futopt = client.futopt;
      });

      test('on method exists and is a function', () => {
        expect(typeof futopt.on).toBe('function');
      });

      test('connect method exists and is a function', () => {
        expect(typeof futopt.connect).toBe('function');
      });

      test('subscribe method exists and is a function', () => {
        expect(typeof futopt.subscribe).toBe('function');
      });

      test('unsubscribe method exists and is a function', () => {
        expect(typeof futopt.unsubscribe).toBe('function');
      });

      test('disconnect method exists and is a function', () => {
        expect(typeof futopt.disconnect).toBe('function');
      });

      test('isConnected is a boolean', () => {
        expect(typeof futopt.isConnected).toBe('boolean');
      });

      test('isClosed is a boolean', () => {
        expect(typeof futopt.isClosed).toBe('boolean');
      });

      test('initial state is disconnected', () => {
        expect(futopt.isConnected).toBe(false);
      });
    });
  });

  describe('Event callback registration', () => {
    test('stock WebSocket accepts event callbacks without error', () => {
      const client = new WebSocketClient('test-key');

      expect(() => {
        client.stock.on('message', (data) => {});
        client.stock.on('connect', (info) => {});
        client.stock.on('disconnect', (reason) => {});
        client.stock.on('reconnect', (info) => {});
        client.stock.on('error', (err) => {});
      }).not.toThrow();
    });

    test('futopt WebSocket accepts event callbacks without error', () => {
      const client = new WebSocketClient('test-key');

      expect(() => {
        client.futopt.on('message', (data) => {});
        client.futopt.on('connect', (info) => {});
        client.futopt.on('disconnect', (reason) => {});
        client.futopt.on('reconnect', (info) => {});
        client.futopt.on('error', (err) => {});
      }).not.toThrow();
    });
  });
});
