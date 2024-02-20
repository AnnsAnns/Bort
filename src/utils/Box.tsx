export default function Box({title, children, className}: {title: string, children: React.ReactNode, className?: string}) {
    return (
    <div className={`
    bg-base-background-color
    ring-box-color-standard
    ring-4
    break-words
    overflow-hidden
    shadow-square
    shadow-box-color-standard/50
    min-h-[120px]
    min-w-[120px]
    flex flex-col
    m-1
    px-3
    pb-3
    sm:max-h-notfullscreen
    text-base-text-color
    ${className}
    `}>
        <div className="
            bg-box-color-standard
            mx-[-12px]
            min-w-max
            min-h-min
            items-end
            text-left
            text-end
            flex
            justify-end
            align-middle
            p-1
            pb-2
            gap-1
            text-base-background-color
        ">
            ➖⏹️❌
        </div>
        <div className="
            sm:overflow-y-auto
            scrollbar-thin
            scrollbar-track-base-background-color
            scrollbar-thumb-box-color-standard">
        <div className="
            font-headerf
            text-2xl
            md:text-3xl
            lg:text-4xl
        ">
            {title}
        </div>
        <div>
            {children}
        </div>
        </div>
    </div>
    )
}