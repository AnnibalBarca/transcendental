import { useTranslation } from "react-i18next";
import { ThemeButton } from "@/features/play/components/ThemeButton";

export default function FtLogin() {
	const { t } = useTranslation();
	const handleClick = () => {
		window.location.href = "/api/auth/42/login";
	};

	return (
		<ThemeButton
			type="button"
			onClick={handleClick}
			texture="/img/carte/beast_of_burden_0.svg"
			texturePosition="center 80%"
			textureZoom={100}
			className="h-[50px] w-full justify-start gap-3 rounded-[2px] pl-[15px]"
		>
			<div className="flex items-center justify-center w-[35px] h-[35px] bg-white rounded-[30%] p-[5px]">
				<img
					className="shrink-0"
					src={`${import.meta.env.VITE_IMAGE_MINIO}/assets/42_icon_black.svg`}
					alt="42"
					width="30"
					height="30"
					onError={(e) => {
						e.currentTarget.style.display = "none";
					}}
				/>
			</div>
			<span>{t("auth.continueWith42")}</span>
		</ThemeButton>
	);
}
