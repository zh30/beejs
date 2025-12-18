export const BeeLogo = ({ className = "w-8 h-8" }: { className?: string }) => (
  <img
    src="/logo.png"
    alt="Beejs Logo"
    className={`${className} object-contain mix-blend-screen`}
  />
);
