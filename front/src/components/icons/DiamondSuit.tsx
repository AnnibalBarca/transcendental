import type { SVGProps } from "react";

const DiamondSuit = (props: SVGProps<SVGSVGElement>) => (
	<svg
		{...props}
		fill={props.fill || "#000000"}
		viewBox="0 0 530 653"
		xmlns="http://www.w3.org/2000/svg"
		width={props.width || 24}
		height={props.height || 24}
	>	
		<path d="M265 1L530 327L265 653L0 327L265 1Z"/>
	</svg>

);

export { DiamondSuit };
