const SideLabels = () => (
  <>
    <div className="light absolute bottom-[2vh] left-[2vw] flex items-center font-medium [writing-mode:vertical-rl] [text-orientation:upright]">
      <span className="mb-3 block h-7 w-[0.05rem] bg-[#f1ede6]" />
      <p className="text-xs tracking-[0.3rem]">四〇〇</p>
    </div>

    <div className="absolute top-[45%] right-[3.3vw] flex -translate-y-1/2 flex-col items-center font-medium">
      <span className="text-[2vw] tracking-[0.3vw] [writing-mode:vertical-rl] [text-orientation:upright]">
        スパイダー
      </span>
      <span className="my-4 block h-12 w-[0.1vw] bg-[#2b2a28]" />
      <span className="text-[1vw] tracking-[0.3vw] [writing-mode:vertical-rl] [text-orientation:mixed]">
        PHANTOM TROUPE
      </span>
    </div>
  </>
);

export default SideLabels;