interface WalletProps {
	balance?: number;
}

export function Wallet({ balance = 0 }: WalletProps) {
	return (
		<div className="inline-flex items-center gap-2 rounded-[2px] border border-[#334155]/60 bg-[#0f172a]/90 px-4 py-2 text-sm font-semibold text-white shadow-[0_4px_20px_rgba(0,0,0,0.4)] backdrop-blur-lg select-none">
			<img
				src={`${import.meta.env.VITE_IMAGE_MINIO}/assets/RoyalCoins.svg`}
				alt="Coin"
				className="h-5 w-5 shrink-0"
			/>
			<span className="min-w-[1ch] text-center">{balance}</span>
		</div>
	);
}
