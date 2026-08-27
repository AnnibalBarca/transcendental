import * as React from "react"
import { Switch as SwitchPrimitive } from "@base-ui/react/switch"

import { cn } from "@/lib/utils"

const IMG_BASE = (import.meta.env.VITE_IMAGE_MINIO as string | undefined) || "/img";
const TEXTURE_BG = `url("${IMG_BASE}/carte/breakthrough_0.svg")`;

function Switch({ className, ...props }: SwitchPrimitive.Root.Props) {
  return (
    <SwitchPrimitive.Root
      data-slot="switch"
      className={cn(
        "peer relative overflow-hidden data-checked:bg-primary data-unchecked:bg-input focus-visible:ring-ring/50 dark:data-unchecked:bg-input/80 inline-flex h-[1.15rem] w-8 shrink-0 items-center rounded-[6px] border border-transparent shadow-xs transition-all outline-none focus-visible:ring-[3px] disabled:cursor-not-allowed disabled:opacity-50",
        className
      )}
      {...props}
    >
      <span
        aria-hidden
        className="pointer-events-none absolute inset-0 mix-blend-screen opacity-40"
        style={{
          backgroundImage: TEXTURE_BG,
          backgroundSize: "300%",
          backgroundPosition: "center 20%",
          filter: "grayscale(100%)",
        }}
      />
      <SwitchPrimitive.Thumb
        data-slot="switch-thumb"
        className={cn(
          "bg-background dark:data-unchecked:bg-foreground/10 pointer-events-none block size-4 rounded-[2px] ring-0 transition-transform data-checked:translate-x-[calc(100%-0.25rem)] data-unchecked:translate-x-0.5"
        )}
      />
    </SwitchPrimitive.Root>
  )
}

export { Switch }