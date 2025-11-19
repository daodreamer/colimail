interface WalletSession {
    address: string;
    chain_id: number;
    connection_method: "walletconnect"; // Only WalletConnect supported for security
    last_active_timestamp: number;
    wc_session_topic?: string;
}
