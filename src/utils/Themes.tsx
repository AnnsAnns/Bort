import Box from "./Box";

export enum Themes {
    standard,
    trans,
    nostalgia,
    ace,
    bi,
    werwolvdark
}

export function ThemeSwitcher() {
  let theme = loadTheme();

  loadSetTheme();

  return (
    <div>
      <a href="#" onClick={
        (e) => {
          e.preventDefault();
          theme = loadTheme();
          const newTheme = theme < lastTheme ? theme+1 : 0;
          localStorage.setItem("theme", newTheme.toString())
          console.log(`Theme: ${Themes[newTheme]}`);

          // Reload the page to apply the new theme
          // This way we can have a static site with dynamic themes
          //location.reload();

          loadSetTheme();

          // Change text within "theme-name" div
          const themeName = document.querySelector('.theme-name');

          if (themeName) {
            themeName.textContent = Themes[newTheme];
          }
        }
      } >
        <Box title="">
            <div className="content-center p-3 text-center">
                <div className="
                  font-headerf
                  text-lg
                  md:text-xl
                  lg:text-2xl
                  bg-base-text-color/25
                  rounded-lg
                  p-3
                ">
                  Change Theme
                </div>
                <div className='theme-name font-light text-base-text-color/40 mb-[-12px]'>
                  {`${Themes[theme]}`}
                </div>
            </div>
        </Box>
      </a>
    </div>
  );

}

// TS Enums have no (good) way of finding the last element to prevent rollovers
// So it's either this or a cursed function
export const lastTheme = Themes.werwolvdark;

export function loadSetTheme() {
  console.log("Loading theme");
  const theme = loadTheme();

  document.documentElement.classList.add(Themes[theme]);

  // Go through each theme and remove the class if it exists
  for (let i = 0; i <= lastTheme; i++) {
    if (Themes[i] === Themes[theme]) {continue;}

    console.log(`Removing ${Themes[i]}`);
    document.documentElement.classList.remove(Themes[i]);
  }
}

export function loadTheme(): number {
    let storedTheme: unknown = null;
    let retTheme = Themes.standard;
  
    if (typeof window !== "undefined") {
      storedTheme = localStorage.getItem("theme");
    }
  
    if (typeof storedTheme === 'number') {
      retTheme = storedTheme;
    } else if (typeof storedTheme === 'string') {
      retTheme = parseInt(storedTheme);
    } else {
      retTheme = Themes.standard;
    }

    console.log(`Theme: ${Themes[retTheme]}`);

    return retTheme;
}