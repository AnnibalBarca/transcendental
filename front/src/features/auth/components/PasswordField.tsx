import HideIcon from '@/components/icons/HideIcon'
import ShowIcon from '@/components/icons/ShowIcon'

interface PasswordFieldProps {
	name: string;
	placeholder: string;
	value?: string;
	onChange?: (e: React.ChangeEvent<HTMLInputElement>) => void;
	isVisible: boolean;
	toggleVisibility: () => void;
	strengthClass?: string;
	minLength?: number;
	maxLength?: number;
}

const INPUT_CLASS =
	"w-full p-2.5 border border-[#334155]/60 bg-[#0f172a]/70 rounded-[2px] text-white text-base focus:outline-none! focus:border-[#60a5fa] focus:border-2";

export default function PasswordField({
	name,
	placeholder,
	value,
	onChange,
	isVisible,
	toggleVisibility,
	strengthClass = "",
	minLength,
	maxLength,
}: PasswordFieldProps) {
{
	return (
		<div className={`relative w-full flex items-center ${strengthClass}`}>
			<input
				className={`${INPUT_CLASS} pr-[42px] ${strengthClass}`}
				onChange={onChange}
				name={name}
				value={value}
				minLength={minLength}
				maxLength={maxLength}
				type={isVisible ? "text" : "password"}
				placeholder={placeholder}
				required
			/>
			<div onClick={toggleVisibility} className="absolute right-2.5 flex items-center justify-center cursor-pointer text-white">
				{isVisible ?
					<HideIcon className="pointer-events-none" color="white" width={25} height={25} /> :
					<ShowIcon className="pointer-events-none" color="white" width={25} height={25} />}
			</div>
		</div>
	);
}
}