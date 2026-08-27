type ButtonProps = {
	children: React.ReactNode
	onClick?: () => void
	disabled?: boolean
	variant?: 'primary' | 'secondary' | 'ghost' | 'unstyled' | 'danger'
	type?: 'button' | 'submit' | 'reset'
	className?: string;
	key?: string | number;
	onAnimationEnd?:  React.AnimationEventHandler<HTMLButtonElement>
	style?: React.CSSProperties;

}

const VARIANT_CLASSES: Record<string, string> = {
	primary: "bg-linear-to-br disabled:text-black from-[#06b6d4] to-[#10b981] text-white shadow-[0_4px_16px_rgba(6,182,212,0.25)] enabled:hover:brightness-110 enabled:hover:shadow-[0_6px_20px_rgba(6,182,212,0.35)] disabled:bg-[rgba(255,255,255,0.08)] disabled:text-[#94a3b8] disabled:shadow-none",
	secondary: "border border-[rgba(255,255,255,0.12)] bg-[rgba(255,255,255,0.06)] text-white backdrop-blur-[8px] enabled:hover:border-[rgba(255,255,255,0.22)] enabled:hover:bg-[rgba(255,255,255,0.12)]",
	ghost: "bg-transparent text-[rgba(255,255,255,0.75)] enabled:hover:bg-[rgba(255,255,255,0.06)] enabled:hover:text-white",
	danger: "border border-[rgba(248,113,113,0.3)] bg-[rgba(248,113,113,0.12)] text-[#f87171] enabled:hover:border-[rgba(248,113,113,0.5)] enabled:hover:bg-[rgba(248,113,113,0.2)]",
};

const BASE_CLASSES =
	"flex w-full cursor-pointer items-center justify-center rounded-xl font-semibold transition-[0.2s] border-none enabled:cursor-pointer disabled:cursor-not-allowed";

function Button(
	{
		children,
		onClick,
		disabled = false,
		variant = 'primary',
		type = 'button',
		className = '',
		onAnimationEnd,
		style,
	}: ButtonProps
)
{
	const computedClassName = variant === 'unstyled'
		? className
		:	`${BASE_CLASSES} ${VARIANT_CLASSES[variant]}
				${className}`.trim();

	return (
		<button
			type={type}
			onClick={onClick}
			disabled={disabled}
			className={computedClassName}
			onAnimationEnd={onAnimationEnd}
			style={style}
		>
			{children}
		</button>
	);
}

export default Button;