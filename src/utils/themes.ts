export enum Themes {
    standard,
    werwolvdark,
    trans,
    nostalgia,
}

// TS Enums have no (good) way of finding the last element to prevent rollovers
// So it's either this or a cursed function
export const lastTheme = Themes.nostalgia;

export function loadSetTheme() {
  const theme = loadTheme();

  document.documentElement.classList.add(Themes[theme]);

  // Go through each theme and remove the class if it exists
  for (let i = 0; i <= lastTheme; i++) {
    if (Themes[i] === Themes[theme]) {continue;}
    document.documentElement.classList.remove(Themes[i]);
  }
}

export function loadTheme(): number {
    let storedTheme: unknown = null;
    let retTheme = Themes.standard;
  
    if (typeof window !== "undefined") {
      storedTheme = localStorage.getItem("theme_v2");
    }
  
    if (typeof storedTheme === 'number') {
      retTheme = storedTheme;
    } else if (typeof storedTheme === 'string') {
      retTheme = parseInt(storedTheme);
    } else {
      retTheme = Themes.standard;
    }

    return retTheme;
}

export function clickLoadTheme(e: Event) {
    e.preventDefault();
    const theme = loadTheme();
    const newTheme = theme < lastTheme ? theme+1 : 0;
    localStorage.setItem("theme_v2", newTheme.toString())

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