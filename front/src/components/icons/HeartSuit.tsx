import type { SVGProps } from "react";

const HeartSuit = (props: SVGProps<SVGSVGElement>) => (
	<svg
		{...props}
		fill={props.fill || "#000000"}
		viewBox="0 0 800 750"
		xmlns="http://www.w3.org/2000/svg"
		width={props.width || 24}
		height={props.height || 24}
	>	
		<path d="M62.132 412.132L400 750L737.87 412.132C777.65 372.349 800 318.393 800 262.132V252.617C800 140.715 709.285 50 597.385 50C535.83 50 477.616 77.9795 439.165 126.043L400 175L360.835 126.043C322.384 77.9795 264.169 50 202.617 50C90.715 50 0 140.715 0 252.617V262.132C0 318.393 22.3495 372.349 62.132 412.132Z"/>
	</svg>

);

export { HeartSuit };
