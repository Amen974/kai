const Home = () => {
  return (
    <div className="corner-bg flex-1 min-h-0 w-full relative">
      <img
        src="/logo.png"
        alt="logo"
        className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 pointer-events-none select-none"
      />
      <div className="absolute top-[62%] left-1/2 -translate-x-1/2 text-xs tracking-widest uppercase opacity-35 select-none whitespace-nowrap">
        ctrl+m compose · ctrl+h history · ctrl+/ help
      </div>
    </div>
  );
};

export default Home;