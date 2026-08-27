import { useTranslation } from "react-i18next";
import {
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuItem,
	DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { ChevronDown } from "lucide-react";

const LANGUAGES = [
	{ code: "fr", label: "Français" },
	{ code: "en", label: "English" },
	{ code: "es", label: "Español" },
	{ code: "sr", label: "Srpski" },
	{ code: "de", label: "Deutsch" },
	{ code: "it", label: "Italiano" },
];

function flagUrl(code: string) {
	return `${import.meta.env.VITE_IMAGE_MINIO}/assets/${code}.svg`;
}

function handleImageError(e: React.SyntheticEvent<HTMLImageElement>) {
	e.currentTarget.style.display = "none";
}

export default function LanguageSelector() {
	const { t, i18n } = useTranslation();
	const current = LANGUAGES.find((l) => l.code === i18n.language) ?? LANGUAGES[0];

	const handleSelect = (code: string) => {
		i18n.changeLanguage(code);
	};

	return (
		<DropdownMenu>
			<DropdownMenuTrigger>
				<span
					className="inline-flex h-10 w-14 cursor-pointer items-center justify-center gap-1.5 rounded-lg border border-[#0b3140] bg-[#101522] text-[#06b6d4] transition-colors duration-200 hover:border-[#06b6d4] hover:bg-[#06b6d4] hover:text-[#080b14] focus-visible:border-[#06b6d4] focus-visible:bg-[#06b6d4] focus-visible:text-[#080b14]"
					aria-label={t("friends.languageSelector", { language: current.label })}
					title={current.label}
				>
					<img
						src={flagUrl(current.code)}
						alt={current.label}
						className="h-auto w-6 rounded-sm pointer-events-none"
						onError={handleImageError}
					/>
					<ChevronDown size={16} className="pointer-events-none" />
				</span>
			</DropdownMenuTrigger>
			<DropdownMenuContent align="end" className="min-w-[140px] rounded-lg border border-[#0b3140] bg-[#101522] p-1.5">
				{LANGUAGES.map((lang) => (
					<DropdownMenuItem
						key={lang.code}
						onClick={() => handleSelect(lang.code)}
						className={
							i18n.language === lang.code
								? "flex cursor-pointer items-center gap-2.5 rounded-md bg-[#06b6d4] px-2.5 py-2 text-[#080b14]"
								: "flex cursor-pointer items-center gap-2.5 rounded-md px-2.5 py-2 text-white transition-colors duration-150 hover:bg-[rgba(6,182,212,0.15)] hover:text-[#06b6d4]"
						}
						title={lang.label}
					>
						<img
							src={flagUrl(lang.code)}
							alt={lang.label}
							className="h-auto w-6 rounded-sm pointer-events-none"
							onError={handleImageError}
						/>
						<span className="pointer-events-none text-[0.85rem] font-semibold uppercase">{lang.code.toUpperCase()}</span>
					</DropdownMenuItem>
				))}
			</DropdownMenuContent>
		</DropdownMenu>
	);
}
