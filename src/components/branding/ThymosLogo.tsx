type ThymosLogoProps = {
  className?: string;
  wordmark?: boolean;
  priority?: boolean;
};

export function ThymosLogo({
  className,
  wordmark = true,
  priority = false,
}: ThymosLogoProps) {
  return (
    <div className={className ? `thymos-logo ${className}` : "thymos-logo"}>
      <img
        className="thymos-mark"
        src="/thymos-mark.png"
        alt=""
        aria-hidden="true"
        width={1024}
        height={1024}
        decoding="async"
        loading="eager"
        fetchPriority={priority ? "high" : "auto"}
      />

      {wordmark ? (
        <span className="thymos-wordmark">
          <strong>THYMOS</strong>
          <span>governed ai runtime</span>
        </span>
      ) : null}
    </div>
  );
}
