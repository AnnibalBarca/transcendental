import LanguageSelector from '@/components/LanguageSelector/LanguageSelector'

const IMG_BASE = (import.meta.env.VITE_IMAGE_MINIO as string | undefined) || "/img";

export default function AuthLayout({ children }: { children: React.ReactNode }) {
	return (
		<div className="relative min-h-screen flex bg-black font-['Libre_Caslon_Text',serif]">
			<img
				src={`${IMG_BASE}/carte/hermit_architect_0.svg`}
				alt=""
				className="pointer-events-none absolute inset-0 h-full w-full object-cover opacity-25"
			/>
			<div className="relative z-10 flex w-full">
				<div className="fixed top-4 right-4 flex items-center gap-2 z-50">
					<LanguageSelector />
				</div>
				{children}
			</div>
		</div>
	);
}