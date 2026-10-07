const SideLabels = () => (
  <div className="pointer-events-none select-none z-layer-decor" aria-hidden="true">
    <div className="light absolute bottom-[2vh] left-[2vw] flex items-center font-medium [writing-mode:vertical-rl] [text-orientation:upright] pointer-events-none">
      <span className="mb-3 block h-7 w-[0.05rem] bg-(--color-bg)" />
      <p className="text-xs tracking-[0.3rem]">四〇〇</p>
    </div>

    <div className="absolute top-[45%] right-[3.3vw] flex -translate-y-1/2 flex-col items-center font-medium pointer-events-none">
      <span className="text-xl tracking-[0.25rem] [writing-mode:vertical-rl] [text-orientation:upright]">
        スパイダー
      </span>
      <span className="my-4 block h-12 w-px bg-(--color-text-main)" />
      <span className="text-xs tracking-[0.2rem] [writing-mode:vertical-rl] [text-orientation:mixed]">
        PHANTOM TROUPE
      </span>
    </div>
  </div>
);

export default SideLabels;