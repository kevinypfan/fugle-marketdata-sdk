package tw.com.fugle.marketdata.generated;


import java.util.List;
import java.util.Map;
import java.util.concurrent.CompletableFuture;
import com.sun.jna.*;
import com.sun.jna.ptr.*;
/**
 * FutOpt intraday endpoints with typed model returns
 */
public interface FutOptIntradayClientInterface {
    
    /**
     * Get available products list (async)
     *
     * typ: "F" for futures, "O" for options
     */
    public CompletableFuture<ProductsResponse> getProducts(String typ) ;
    
    /**
     * Get quote for a futures/options contract (async)
     *
     * after_hours: true for after-hours session
     */
    public CompletableFuture<FutOptQuote> getQuote(String symbol, Boolean afterHours) ;
    
    /**
     * Get ticker info for a contract (async)
     */
    public CompletableFuture<FutOptTicker> getTicker(String symbol, Boolean afterHours) ;
    
    /**
     * Get available products list (sync/blocking)
     */
    public ProductsResponse productsSync(String typ) throws MarketDataException;
    
    /**
     * Get quote for a futures/options contract (sync/blocking)
     */
    public FutOptQuote quoteSync(String symbol, Boolean afterHours) throws MarketDataException;
    
    /**
     * Get ticker info for a contract (sync/blocking)
     */
    public FutOptTicker tickerSync(String symbol, Boolean afterHours) throws MarketDataException;
    
}

