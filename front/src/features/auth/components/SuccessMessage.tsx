import { type ReactNode } from "react";
import { useNavigate } from "react-router-dom";
import { useTranslation } from "react-i18next";

interface SuccessMessageProps {
	icon: ReactNode;
	title: string;
	message: string;
	countdown?: number;
	buttonText?: string;
	onButtonClick?: () => void;
	showCountdown?: boolean;
}

export default function SuccessMessage({
	icon,
	title,
	message,
	countdown,
	buttonText,
	onButtonClick,
	showCountdown = true,
}: SuccessMessageProps) {
	const { t } = useTranslation();
	const navigate = useNavigate();

	const handleClick = () => {
		if (onButtonClick) {
			onButtonClick();
		} else {
			navigate("/login");
		}
	};

	return (
		<div className="flex flex-col items-center justify-center gap-[30px] w-full max-w-[400px]">
			<div className="w-20 h-20 flex items-center justify-center animate-[auth-scale-in_0.6s_ease-out] drop-shadow-[0_8px_32px_rgba(6,182,212,0.3)]">{icon}</div>

			<div>
				<h2 className="text-[28px] font-bold text-white text-center m-0">{title}</h2>
				<p className="text-base text-[#94a3b8] text-center leading-[1.6] m-0">{message}</p>
				{showCountdown && countdown !== undefined && (
					<p className="text-sm text-[#64748b] text-center mt-5">
						{t("password.redirectToLogin", { countdown })}
					</p>
				)}
			</div>

			<button
				className="mt-5 px-[30px] py-2.5 bg-[#06b6d4] text-white rounded-lg font-bold cursor-pointer uppercase text-sm transition-all duration-300 hover:bg-[#0891b2] hover:[transform:translateY(-2px)] hover:shadow-[0_4px_12px_rgba(6,182,212,0.4)] active:[transform:translateY(0)]"
				onClick={handleClick}
			>
				{buttonText ?? t("password.goToLogin")}
			</button>
		</div>
	);
}