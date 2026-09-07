export const BeeLogo = ({ className = 'w-8 h-8' }: { className?: string }) => (
  <img
    src="/logo.png"
    alt="Beejs Logo"
    className={`${className} object-contain dark:mix-blend-screen opacity-95 drop-shadow-sm dark:drop-shadow-[0_0_12px_rgba(103,209,255,0.35)]`}
  />
)
